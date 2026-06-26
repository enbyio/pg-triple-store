use std::collections::BTreeMap;

use diesel::{QueryDsl, RunQueryDsl};
use spargebra::algebra::GraphPattern;
use spargebra::term::TriplePattern;
use spargebra::{Query, SparqlParser};

use crate::db::models::property::LiteralMatchMode;
use crate::db::models::query::{QueryOptions, QueryResult, Solution, Term};
use crate::db::models::triple::{TriplePosition, TripleQuery};
use crate::store::TripleStore;
use crate::StoreError::{self, UnsupportedInputData};

impl TripleStore {
    pub fn parse_sparql_query(&mut self, sparql: &str) -> Result<QueryResult, StoreError> {
        let parser = SparqlParser::new();
        let query = parser.parse_query(&self.inject_prefixes(sparql)?.to_string())?;
        match query {
            Query::Select { pattern, .. } => self.execute_pattern(pattern, QueryOptions::default()),
            Query::Construct { .. } => Err(UnsupportedInputData),
            Query::Describe { .. } => Err(UnsupportedInputData),
            Query::Ask { .. } => Err(UnsupportedInputData),
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

    fn execute_pattern(
        &mut self,
        pattern: GraphPattern,
        options: QueryOptions,
    ) -> Result<QueryResult, StoreError> {
        log::debug!("GraphPattern: {}", pattern);
        match pattern {
            GraphPattern::Bgp { patterns } => {
                Ok(QueryResult::Solutions(self.execute_bgp(patterns, options)?))
            }
            GraphPattern::Project { inner, .. } => self.execute_pattern(*inner, options),
            GraphPattern::Distinct { inner } => self.execute_pattern(*inner, options),
            GraphPattern::Slice {
                inner,
                start,
                length,
            } => {
                let new_mods = QueryOptions {
                    offset: options.offset.saturating_add(start),
                    limit: match (options.limit, length) {
                        (None, Some(l)) => Some(l),
                        (Some(existing), Some(l)) => Some(existing.min(l)),
                        (Some(existing), None) => Some(existing),
                        (None, None) => None,
                    },
                };
                self.execute_pattern(*inner, new_mods)
            }
            GraphPattern::OrderBy { inner, .. } => self.execute_pattern(*inner, options),
            GraphPattern::Filter { inner, .. } => self.execute_pattern(*inner, options),
            _ => Err(UnsupportedInputData),
        }
    }

    pub(crate) fn execute_triple_pattern_with_bindings(
        &mut self,
        tp: TriplePattern,
        known: &BTreeMap<String, Vec<Term>>, // var name → allowed values (IN list)
    ) -> Result<Vec<Solution>, StoreError> {
        log::debug!("known variables: {:?}", known);
        log::debug!("Pattern: {:?}", tp);
        let mut query = TripleQuery::new();
        match tp.subject {
            spargebra::term::TermPattern::NamedNode(named_node) => query.subject(
                TriplePosition::Constant(self.normalize_iri(named_node.as_str())?),
            ),
            spargebra::term::TermPattern::Variable(variable) => {
                let name = variable.as_str().to_string();
                if let Some(values) = known.get(&name) {
                    query.subject(TriplePosition::Bound(name, values.clone()));
                } else {
                    query.subject(TriplePosition::Variable(name))
                }
            }
            _ => {
                return Err(StoreError::DataError(
                    "Blank Node, Literal or Triple not supported as subject".to_string(),
                ));
            }
        };
        match tp.predicate {
            spargebra::term::NamedNodePattern::NamedNode(named_node) => query.predicate(
                TriplePosition::Constant(self.normalize_iri(named_node.as_str())?),
            ),
            spargebra::term::NamedNodePattern::Variable(variable) => {
                let name = variable.as_str().to_string();
                if let Some(values) = known.get(&name) {
                    query.predicate(TriplePosition::Bound(name, values.clone()));
                } else {
                    query.predicate(TriplePosition::Variable(name))
                }
            }
        };
        let mut results: Vec<Solution> = Vec::new();
        match tp.object {
            spargebra::term::TermPattern::NamedNode(named_node) => {
                query.object(TriplePosition::Constant(
                    self.normalize_iri(named_node.as_str())?,
                ));
                results.extend(self.query_relation_triples_joined(
                    query.relation_query()?,
                    QueryOptions::default(),
                )?);
            }
            spargebra::term::TermPattern::Literal(literal) => {
                query.object(TriplePosition::Constant(literal.value().to_string()));
                results.extend(self.query_property_triples_joined(
                    query.property_query(LiteralMatchMode::Exact)?,
                    QueryOptions::default(),
                )?);
            }
            spargebra::term::TermPattern::Variable(variable) => {
                let name = variable.as_str().to_string();
                if let Some(values) = known.get(&name) {
                    query.object(TriplePosition::Bound(name, values.clone()));
                } else {
                    query.object(TriplePosition::Variable(name))
                }
                results.extend(self.query_relation_triples_joined(
                    query.relation_query()?,
                    QueryOptions::default(),
                )?);
                results.extend(self.query_property_triples_joined(
                    query.property_query(LiteralMatchMode::Exact)?,
                    QueryOptions::default(),
                )?);
            }
            _ => {
                return Err(StoreError::DataError(
                    "Blank Node or Triple are not supported as objects".to_string(),
                ));
            }
        };
        log::debug!("result: {:?}", results);
        Ok(results)
    }

    pub(crate) fn execute_triple_pattern(
        &mut self,
        tp: TriplePattern,
        options: QueryOptions,
    ) -> Result<Vec<Solution>, StoreError> {
        let mut query = TripleQuery::new();
        match tp.subject {
            spargebra::term::TermPattern::NamedNode(named_node) => query.subject(
                TriplePosition::Constant(self.normalize_iri(named_node.as_str())?),
            ),
            spargebra::term::TermPattern::Variable(variable) => {
                query.subject(TriplePosition::Variable(variable.as_str().to_string()))
            }
            _ => {
                return Err(StoreError::DataError(
                    "Blank Node, Literal or Triple not supported as subject".to_string(),
                ));
            }
        };
        match tp.predicate {
            spargebra::term::NamedNodePattern::NamedNode(named_node) => query.predicate(
                TriplePosition::Constant(self.normalize_iri(named_node.as_str())?),
            ),
            spargebra::term::NamedNodePattern::Variable(variable) => {
                query.predicate(TriplePosition::Variable(variable.as_str().to_string()))
            }
        };
        let mut results: Vec<Solution> = Vec::new();
        match tp.object {
            spargebra::term::TermPattern::NamedNode(named_node) => {
                query.object(TriplePosition::Constant(
                    self.normalize_iri(named_node.as_str())?,
                ));
                results
                    .extend(self.query_relation_triples_joined(query.relation_query()?, options)?);
            }
            spargebra::term::TermPattern::Literal(literal) => {
                query.object(TriplePosition::Constant(literal.value().to_string()));
                results.extend(self.query_property_triples_joined(
                    query.property_query(LiteralMatchMode::Exact)?,
                    options,
                )?);
            }
            spargebra::term::TermPattern::Variable(variable) => {
                query.object(TriplePosition::Variable(variable.as_str().to_string()));
                results
                    .extend(self.query_relation_triples_joined(query.relation_query()?, options)?);
                results.extend(self.query_property_triples_joined(
                    query.property_query(LiteralMatchMode::Exact)?,
                    options,
                )?);
            }
            _ => {
                return Err(StoreError::DataError(
                    "Blank Node or Triple are not supported as objects".to_string(),
                ));
            }
        }
        if let Some(lim) = options.limit {
            Ok(results[0..lim].to_vec())
        } else {
            Ok(results)
        }
    }
}
