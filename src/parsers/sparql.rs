use std::collections::{BTreeMap, HashMap, HashSet};

use diesel::{QueryDsl, RunQueryDsl};
use oxrdf::{BlankNode, Literal, NamedNode, NamedOrBlankNode, Term as OxTerm, Triple, Variable};
use spargebra::algebra::GraphPattern;
use spargebra::term::{NamedNodePattern, TermPattern, TriplePattern};
use spargebra::{Query, SparqlParser};

use crate::error::StoreError;
use crate::model::triple::{LiteralMatchMode, Term, TriplePosition, VarKey};
use crate::query::solution::{QueryResult, Solution, SolutionSet};
use crate::query::triples::{query_property_triples, query_relation_triples};
use crate::store::TripleStore;

macro_rules! continue_on_err {
    ($result:expr) => {
        match $result {
            Ok(val) => val,
            Err(err) => {
                log::error!("Error: {:?}", err);
                continue;
            }
        }
    };
}

impl TripleStore {
    pub(crate) fn parse_sparql_query(&self, sparql: &str) -> Result<QueryResult, StoreError> {
        let parser = SparqlParser::new();
        let prefix_injected_query = self.inject_prefixes(sparql)?;
        let query = parser.parse_query(&prefix_injected_query)?;
        match query {
            Query::Select { pattern, .. } => self.execute_pattern(pattern),
            Query::Construct {
                template, pattern, ..
            } => self.execute_construct(template, pattern),
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

    pub(crate) fn execute_pattern(&self, pattern: GraphPattern) -> Result<QueryResult, StoreError> {
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

    fn execute_construct(
        &self,
        template: Vec<TriplePattern>,
        pattern: GraphPattern,
    ) -> Result<QueryResult, StoreError> {
        let QueryResult::Solutions(solutions) = self.execute_pattern(pattern)? else {
            return Err(StoreError::sparql_error(
                "Construct requires a solution pattern to work",
            ));
        };
        let mut triples: HashSet<Triple> = HashSet::new();

        for row in solutions.rows {
            for triple in self.instantiate(&template, &row) {
                triples.insert(triple);
            }
        }

        Ok(QueryResult::Graph(triples.into_iter().collect()))
    }

    fn instantiate(&self, patterns: &[TriplePattern], row: &Solution) -> Vec<Triple> {
        let mut triples: Vec<Triple> = Vec::new();

        let mut bnodes: HashMap<String, BlankNode> = HashMap::new();

        for tp in patterns {
            let subject = match &tp.subject {
                TermPattern::NamedNode(nn) => NamedOrBlankNode::NamedNode(continue_on_err!(
                    self.normalize_iri(nn.as_str())
                        .and_then(|iri| NamedNode::new(iri).map_err(StoreError::from))
                )),
                TermPattern::BlankNode(bn) => NamedOrBlankNode::BlankNode(
                    bnodes.entry(bn.as_str().to_string()).or_default().clone(),
                ),
                TermPattern::Variable(var) => match row.get(var.as_str()) {
                    Some(Term::Iri(iri)) => {
                        let node = continue_on_err!(
                            self.normalize_iri(iri)
                                .and_then(|iri| NamedNode::new(iri).map_err(StoreError::from))
                        );
                        NamedOrBlankNode::NamedNode(node)
                    }
                    Some(Term::BlankNode(bnode)) => {
                        let node =
                            continue_on_err!(BlankNode::new(bnode).map_err(StoreError::from));
                        NamedOrBlankNode::BlankNode(node)
                    }
                    _ => continue,
                },
                _ => continue, // Literal / Triple are not valid as subjects
            };
            let predicate = match &tp.predicate {
                NamedNodePattern::NamedNode(nn) => continue_on_err!(
                    self.normalize_iri(nn.as_str())
                        .and_then(|iri| NamedNode::new(iri).map_err(StoreError::from))
                ),
                NamedNodePattern::Variable(var) => match row.get(var.as_str()) {
                    Some(Term::Iri(iri)) => continue_on_err!(NamedNode::new(iri.clone())),
                    _ => continue,
                },
            };
            let object: OxTerm = match &tp.object {
                TermPattern::NamedNode(nn) => OxTerm::NamedNode(continue_on_err!(
                    self.normalize_iri(nn.as_str())
                        .and_then(|iri| NamedNode::new(iri).map_err(StoreError::from))
                )),
                TermPattern::BlankNode(bn) => {
                    OxTerm::BlankNode(bnodes.entry(bn.as_str().to_string()).or_default().clone())
                }
                TermPattern::Literal(lit) => OxTerm::Literal(lit.clone()),
                TermPattern::Variable(var) => match row.get(var.as_str()) {
                    Some(Term::Iri(iri)) => {
                        let node = continue_on_err!(
                            self.normalize_iri(iri)
                                .and_then(|iri| NamedNode::new(iri).map_err(StoreError::from))
                        );
                        OxTerm::NamedNode(node)
                    }
                    Some(Term::BlankNode(bnode)) => {
                        let node =
                            continue_on_err!(BlankNode::new(bnode).map_err(StoreError::from));
                        OxTerm::BlankNode(node)
                    }
                    Some(Term::Literal { value, datatype }) => {
                        let lit_type = continue_on_err!(NamedNode::new(datatype));
                        OxTerm::Literal(Literal::new_typed_literal(value, lit_type))
                    }
                    None => continue,
                },
                _ => continue, // rdf star not supported for now
            };
            triples.push(Triple::new(subject, predicate, object));
        }

        triples
    }

    pub(crate) fn execute_triple_pattern_with_bindings(
        &self,
        tp: TriplePattern,
        known: &BTreeMap<VarKey, Vec<Term>>, // var name → allowed values (IN list)
    ) -> Result<Vec<Solution>, StoreError> {
        log::debug!("known variables: {:?}", known);
        log::debug!("Pattern: {:?}", tp);
        let subject = match tp.subject {
            TermPattern::NamedNode(named_node) => {
                TriplePosition::Constant(self.normalize_iri(named_node.as_str())?)
            }
            TermPattern::Variable(variable) => {
                let name = VarKey::Named(variable.as_str().to_string());
                if let Some(values) = known.get(&name) {
                    TriplePosition::Bound(name, values.clone())
                } else {
                    TriplePosition::Variable(name)
                }
            }
            TermPattern::BlankNode(bnode) => {
                let key = VarKey::Blank(bnode.as_str().to_string());
                if let Some(values) = known.get(&key) {
                    TriplePosition::Bound(key, values.clone())
                } else {
                    TriplePosition::Variable(key)
                }
            }
            _ => {
                return Err(StoreError::sparql_error(
                    "Literal or Triple not supported as subject",
                ));
            }
        };
        let predicate = match tp.predicate {
            NamedNodePattern::NamedNode(named_node) => {
                TriplePosition::Constant(self.normalize_iri(named_node.as_str())?)
            }
            NamedNodePattern::Variable(variable) => {
                let key = VarKey::Named(variable.as_str().to_string());
                if let Some(values) = known.get(&key) {
                    TriplePosition::Bound(key, values.clone())
                } else {
                    TriplePosition::Variable(key)
                }
            }
        };
        let mut results: Vec<Solution> = Vec::new();
        let mut conn = self.conn()?;
        match tp.object {
            TermPattern::NamedNode(named_node) => {
                let object = TriplePosition::Constant(self.normalize_iri(named_node.as_str())?);
                results.extend(query_relation_triples(
                    &mut conn, subject, predicate, object,
                )?);
            }
            TermPattern::Literal(literal) => {
                let object = TriplePosition::Constant(literal.value().to_string());
                results.extend(query_property_triples(
                    &mut conn,
                    subject,
                    predicate,
                    object,
                    LiteralMatchMode::Exact,
                )?);
            }
            TermPattern::Variable(variable) => {
                let key = VarKey::Named(variable.as_str().to_string());
                let object = if let Some(values) = known.get(&key) {
                    TriplePosition::Bound(key, values.clone())
                } else {
                    TriplePosition::Variable(key)
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
            TermPattern::BlankNode(bnode) => {
                let key = VarKey::Blank(bnode.as_str().to_string());
                let object = if let Some(values) = known.get(&key) {
                    TriplePosition::Bound(key, values.clone())
                } else {
                    TriplePosition::Variable(key)
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
                    "Triple are not supported as objects",
                ));
            }
        };
        log::debug!("result: {:?}", results);
        Ok(results)
    }

    pub(crate) fn execute_triple_pattern(
        &self,
        tp: TriplePattern,
    ) -> Result<Vec<Solution>, StoreError> {
        let mut conn = self.conn()?;
        let subject = match tp.subject {
            TermPattern::NamedNode(named_node) => {
                TriplePosition::Constant(self.normalize_iri(named_node.as_str())?)
            }
            TermPattern::Variable(variable) => {
                TriplePosition::Variable(VarKey::Named(variable.as_str().to_string()))
            }
            TermPattern::BlankNode(bnode) => {
                TriplePosition::Variable(VarKey::Blank(bnode.as_str().to_string()))
            }
            _ => {
                return Err(StoreError::data_error(
                    "Literal or Triple not supported as subject",
                ));
            }
        };
        let predicate = match tp.predicate {
            NamedNodePattern::NamedNode(named_node) => {
                TriplePosition::Constant(self.normalize_iri(named_node.as_str())?)
            }
            NamedNodePattern::Variable(variable) => {
                TriplePosition::Variable(VarKey::Named(variable.as_str().to_string()))
            }
        };
        let mut results: Vec<Solution> = Vec::new();
        match tp.object {
            TermPattern::NamedNode(named_node) => {
                let object = TriplePosition::Constant(self.normalize_iri(named_node.as_str())?);
                results.extend(query_relation_triples(
                    &mut conn, subject, predicate, object,
                )?);
            }
            TermPattern::Literal(literal) => {
                let object = TriplePosition::Constant(literal.value().to_string());
                results.extend(query_property_triples(
                    &mut conn,
                    subject,
                    predicate,
                    object,
                    LiteralMatchMode::Exact,
                )?);
            }
            TermPattern::Variable(variable) => {
                let object = TriplePosition::Variable(VarKey::Named(variable.as_str().to_string()));
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
            TermPattern::BlankNode(variable) => {
                let object = TriplePosition::Variable(VarKey::Blank(variable.as_str().to_string()));
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
                    "Triples are not supported as objects",
                ));
            }
        }
        // if let Some(lim) = options.limit {
        //     Ok(results[0..lim].to_vec())
        // } else {
        Ok(results)
        // }
    }

    fn execute_bgp(&self, patterns: Vec<TriplePattern>) -> Result<SolutionSet, StoreError> {
        if patterns.is_empty() {
            return Ok(SolutionSet {
                vars: vec![],
                rows: vec![],
            });
        }
        let mut acc: Vec<Solution> = vec![];
        let mut acc_vars: HashSet<VarKey> = HashSet::new();
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

            let shared: HashSet<&VarKey> = pattern_vars.intersection(&acc_vars).collect();
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
                    let known: BTreeMap<VarKey, Vec<Term>> = shared
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
                let known_values: BTreeMap<VarKey, Vec<Term>> = shared
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

        let mut vars: Vec<String> = acc_vars
            .into_iter()
            .filter(VarKey::is_named)
            .map(VarKey::into_name)
            .collect();
        vars.sort();

        Ok(SolutionSet { rows: acc, vars })
    }

    fn project_pattern(
        &self,
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

pub(crate) fn triple_pattern_vars(pattern: &TriplePattern) -> HashSet<VarKey> {
    let mut set: HashSet<VarKey> = HashSet::new();
    match &pattern.subject {
        TermPattern::Variable(var) => {
            set.insert(VarKey::Named(var.as_str().to_string()));
        }
        TermPattern::BlankNode(bnode) => {
            set.insert(VarKey::Blank(bnode.as_str().to_string()));
        }
        _ => {}
    }
    if let NamedNodePattern::Variable(var) = &pattern.predicate {
        set.insert(VarKey::Named(var.as_str().to_string()));
    }
    match &pattern.object {
        TermPattern::Variable(var) => {
            set.insert(VarKey::Named(var.as_str().to_string()));
        }
        TermPattern::BlankNode(bnode) => {
            set.insert(VarKey::Blank(bnode.as_str().to_string()));
        }
        _ => {}
    }
    set
}

pub(crate) fn merge(left: &Solution, right: &Solution) -> Solution {
    let mut bindings = left.bindings.clone();
    bindings.extend(right.bindings.clone());
    Solution { bindings }
}
