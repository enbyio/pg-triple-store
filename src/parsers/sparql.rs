use std::collections::{BTreeMap, HashMap, HashSet};

use diesel::{QueryDsl, RunQueryDsl};
use oxrdf::Variable;
use spargebra::algebra::GraphPattern;
use spargebra::term::{NamedNodePattern, TermPattern, TriplePattern};
use spargebra::{Query, SparqlParser};

use crate::error::StoreError;
use crate::model::triple::{LiteralMatchMode, Term, TriplePosition};
use crate::query::solution::{QueryResult, Solution, SolutionSet};
use crate::query::triples::{query_property_triples, query_relation_triples};
use crate::store::TripleStore;

impl TripleStore {
    pub(crate) fn parse_sparql_query(&mut self, sparql: &str) -> Result<QueryResult, StoreError> {
        let parser = SparqlParser::new();
        let prefix_injected_query = self.inject_prefixes(sparql)?;
        let query = parser.parse_query(&prefix_injected_query)?;
        match query {
            Query::Select { pattern, .. } => self.execute_pattern(pattern),
            Query::Construct { .. } => {
                Err(StoreError::sparql_error("Construct is not yet supported"))
            }
            Query::Describe { .. } => {
                Err(StoreError::sparql_error("Describe is not yet supported"))
            }
            Query::Ask { .. } => Err(StoreError::sparql_error("Ask is not yet supported")),
        }
    }

    fn inject_prefixes(&self, query: &str) -> Result<String, StoreError> {
        use crate::schema::prefixes::dsl::*;
        let mut conn = self.conn()?;
        let stored: Vec<(String, String)> = prefixes
            .select((prefix, namespace))
            .load::<(String, String)>(&mut conn)?;

        let prefix_block: String = stored
            .iter()
            .map(|(p, ns)| format!("PREFIX {}: <{}>\n", p, ns))
            .collect();

        Ok(format!("{}{}", prefix_block, query))
    }

    pub(crate) fn execute_pattern(
        &mut self,
        pattern: GraphPattern,
    ) -> Result<QueryResult, StoreError> {
        log::debug!("GraphPattern: {}", pattern);
        match pattern {
            GraphPattern::Bgp { patterns } => {
                Ok(QueryResult::Solutions(self.execute_bgp(patterns)?))
            }
            GraphPattern::Project { inner, variables } => self.project_pattern(*inner, variables),
            GraphPattern::Distinct { inner } => self.execute_pattern(*inner),
            GraphPattern::Slice {
                inner,
                ..
                //start,
                //length,
            } => self.execute_pattern(*inner),
            GraphPattern::OrderBy { inner, .. } => self.execute_pattern(*inner),
            GraphPattern::Filter { inner, expr } => self.add_filter(*inner, expr),
            _ => Err(StoreError::sparql_error(
                "GraphPattern is not yet supported",
            )),
        }
    }

    pub(crate) fn execute_triple_pattern_with_bindings(
        &mut self,
        tp: TriplePattern,
        known: &BTreeMap<String, Vec<Term>>, // var name → allowed values (IN list)
    ) -> Result<Vec<Solution>, StoreError> {
        log::debug!("known variables: {:?}", known);
        log::debug!("Pattern: {:?}", tp);
        let subject = match tp.subject {
            spargebra::term::TermPattern::NamedNode(named_node) => {
                TriplePosition::Constant(self.normalize_iri(named_node.as_str())?)
            }
            spargebra::term::TermPattern::Variable(variable) => {
                let name = variable.as_str().to_string();
                if let Some(values) = known.get(&name) {
                    TriplePosition::Bound(name, values.clone())
                } else {
                    TriplePosition::Variable(name)
                }
            }
            _ => {
                return Err(StoreError::sparql_error(
                    "Blank Node, Literal or Triple not supported as subject",
                ));
            }
        };
        let predicate = match tp.predicate {
            spargebra::term::NamedNodePattern::NamedNode(named_node) => {
                TriplePosition::Constant(self.normalize_iri(named_node.as_str())?)
            }
            spargebra::term::NamedNodePattern::Variable(variable) => {
                let name = variable.as_str().to_string();
                if let Some(values) = known.get(&name) {
                    TriplePosition::Bound(name, values.clone())
                } else {
                    TriplePosition::Variable(name)
                }
            }
        };
        let mut results: Vec<Solution> = Vec::new();
        let mut conn = self.conn()?;
        match tp.object {
            spargebra::term::TermPattern::NamedNode(named_node) => {
                let object = TriplePosition::Constant(self.normalize_iri(named_node.as_str())?);
                results.extend(query_relation_triples(
                    &mut conn, subject, predicate, object,
                )?);
            }
            spargebra::term::TermPattern::Literal(literal) => {
                let object = TriplePosition::Constant(literal.value().to_string());
                results.extend(query_property_triples(
                    &mut conn,
                    subject,
                    predicate,
                    object,
                    LiteralMatchMode::Exact,
                )?);
            }
            spargebra::term::TermPattern::Variable(variable) => {
                let name = variable.as_str().to_string();
                let object = if let Some(values) = known.get(&name) {
                    TriplePosition::Bound(name, values.clone())
                } else {
                    TriplePosition::Variable(name)
                };
                results.extend(query_relation_triples(
                    &mut conn,
                    subject.clone(),
                    predicate.clone(),
                    object.clone(),
                )?);
                results.extend(query_property_triples(
                    &mut conn,
                    subject,
                    predicate,
                    object,
                    LiteralMatchMode::Exact,
                )?);
            }
            _ => {
                return Err(StoreError::data_error(
                    "Blank Node or Triple are not supported as objects",
                ));
            }
        };
        log::debug!("result: {:?}", results);
        Ok(results)
    }

    pub(crate) fn execute_triple_pattern(
        &mut self,
        tp: TriplePattern,
    ) -> Result<Vec<Solution>, StoreError> {
        let mut conn = self.conn()?;
        let subject = match tp.subject {
            spargebra::term::TermPattern::NamedNode(named_node) => {
                TriplePosition::Constant(self.normalize_iri(named_node.as_str())?)
            }
            spargebra::term::TermPattern::Variable(variable) => {
                TriplePosition::Variable(variable.as_str().to_string())
            }
            _ => {
                return Err(StoreError::data_error(
                    "Blank Node, Literal or Triple not supported as subject",
                ));
            }
        };
        let predicate = match tp.predicate {
            spargebra::term::NamedNodePattern::NamedNode(named_node) => {
                TriplePosition::Constant(self.normalize_iri(named_node.as_str())?)
            }
            spargebra::term::NamedNodePattern::Variable(variable) => {
                TriplePosition::Variable(variable.as_str().to_string())
            }
        };
        let mut results: Vec<Solution> = Vec::new();
        match tp.object {
            spargebra::term::TermPattern::NamedNode(named_node) => {
                let object = TriplePosition::Constant(self.normalize_iri(named_node.as_str())?);
                results.extend(query_relation_triples(
                    &mut conn, subject, predicate, object,
                )?);
            }
            spargebra::term::TermPattern::Literal(literal) => {
                let object = TriplePosition::Constant(literal.value().to_string());
                results.extend(query_property_triples(
                    &mut conn,
                    subject,
                    predicate,
                    object,
                    LiteralMatchMode::Exact,
                )?);
            }
            spargebra::term::TermPattern::Variable(variable) => {
                let object = TriplePosition::Variable(variable.as_str().to_string());
                results.extend(query_relation_triples(
                    &mut conn,
                    subject.clone(),
                    predicate.clone(),
                    object.clone(),
                )?);
                results.extend(query_property_triples(
                    &mut conn,
                    subject,
                    predicate,
                    object,
                    LiteralMatchMode::Exact,
                )?);
            }
            _ => {
                return Err(StoreError::data_error(
                    "Blank Node or Triple are not supported as objects",
                ));
            }
        }
        // if let Some(lim) = options.limit {
        //     Ok(results[0..lim].to_vec())
        // } else {
        Ok(results)
        // }
    }

    fn execute_bgp(&mut self, patterns: Vec<TriplePattern>) -> Result<SolutionSet, StoreError> {
        if patterns.is_empty() {
            return Ok(SolutionSet {
                vars: vec![],
                rows: vec![],
            });
        }
        let mut acc: Vec<Solution> = vec![];
        let mut acc_vars: HashSet<String> = HashSet::new();
        let mut seeded = false;

        for pattern in patterns {
            log::debug!("acc: {:?}", acc);
            let pattern_vars = triple_pattern_vars(&pattern);

            if !seeded {
                let rows = self.execute_triple_pattern(pattern)?;
                acc = rows;
                acc_vars = pattern_vars;
                seeded = true;
                continue;
            }

            if acc.is_empty() {
                break;
            }

            let shared: HashSet<&String> = pattern_vars.intersection(&acc_vars).collect();
            if pattern_vars.is_empty() {
                // no variables in pattern, constant existence check
                if !self.execute_triple_pattern(pattern)?.is_empty() {
                    acc = vec![]
                }
            } else if shared.is_empty() {
                let new_rows = self.execute_triple_pattern(pattern)?;
                acc = acc
                    .iter()
                    .flat_map(|l| new_rows.iter().map(move |r| merge(l, r)))
                    .collect();
                acc_vars.extend(pattern_vars);
            } else if shared.len() == pattern_vars.len() {
                acc.retain(|acc_row| {
                    // build known bindings from this acc row for the shared vars
                    let known: BTreeMap<String, Vec<Term>> = shared
                        .iter()
                        .filter_map(|v| {
                            acc_row
                                .bindings
                                .get(*v)
                                .map(|t| ((*v).clone(), vec![t.clone()]))
                        })
                        .collect();
                    self.execute_triple_pattern_with_bindings(pattern.clone(), &known)
                        .map(|rows| !rows.is_empty())
                        .unwrap_or(false)
                });
            } else {
                let known_values: BTreeMap<String, Vec<Term>> = shared
                    .iter()
                    .map(|&v| {
                        let vals: Vec<Term> = acc
                            .iter()
                            .filter_map(|s| s.bindings.get(v).cloned())
                            .collect::<HashSet<_>>() // dedup
                            .into_iter()
                            .collect();
                        ((*v).clone(), vals)
                    })
                    .collect();
                // scan pattern with IN-list filters applied in SQL
                let new_rows = self.execute_triple_pattern_with_bindings(pattern, &known_values)?;

                // hash-join: build index on new_rows keyed by shared var values
                let mut index: HashMap<Vec<Option<Term>>, Vec<Solution>> = HashMap::new();
                for row in new_rows {
                    let key: Vec<Option<Term>> = shared
                        .iter()
                        .map(|v| row.bindings.get(*v).cloned())
                        .collect();
                    index.entry(key).or_default().push(row);
                }

                // probe acc against index
                acc = acc
                    .into_iter()
                    .flat_map(|acc_row| {
                        let key: Vec<Option<Term>> = shared
                            .iter()
                            .map(|v| acc_row.bindings.get(*v).cloned())
                            .collect();
                        match index.get(&key) {
                            None => vec![],
                            Some(matches) => matches.iter().map(|r| merge(&acc_row, r)).collect(),
                        }
                    })
                    .collect();

                acc_vars.extend(pattern_vars);
            }
        }

        let mut vars: Vec<String> = acc_vars.into_iter().collect();
        vars.sort();

        Ok(SolutionSet { rows: acc, vars })
    }

    fn project_pattern(
        &mut self,
        pattern: GraphPattern,
        variables: Vec<Variable>,
    ) -> Result<QueryResult, StoreError> {
        let result = self.execute_pattern(pattern)?;
        match result {
            QueryResult::Solutions(solution_set) => {
                let vars = variables
                    .iter()
                    .map(|var| var.as_str().to_string())
                    .collect();
                Ok(QueryResult::Solutions(SolutionSet {
                    vars,
                    rows: solution_set.rows,
                }))
            }
            QueryResult::Boolean(_) => Err(StoreError::data_error(
                "Project is invalid for boolean results",
            )),
            QueryResult::Graph(_) => Err(StoreError::data_error(
                "Project is invalid for Graph results",
            )),
        }
    }
}

pub(crate) fn triple_pattern_vars(pattern: &TriplePattern) -> HashSet<String> {
    let mut set: HashSet<String> = HashSet::new();
    if let TermPattern::Variable(var) = &pattern.subject {
        set.insert(var.as_str().to_string());
    }
    if let NamedNodePattern::Variable(var) = &pattern.predicate {
        set.insert(var.as_str().to_string());
    }
    if let TermPattern::Variable(var) = &pattern.object {
        set.insert(var.as_str().to_string());
    }
    set
}

pub(crate) fn merge(left: &Solution, right: &Solution) -> Solution {
    let mut bindings = left.bindings.clone();
    bindings.extend(right.bindings.clone());
    Solution { bindings }
}
