use diesel::{QueryDsl, RunQueryDsl};
use spargebra::algebra::GraphPattern;
use spargebra::term::TriplePattern;
use spargebra::{Query, SparqlParser};

use crate::db::models::property::{LiteralMatchMode, PropertyTripleQuery};
use crate::db::models::query::QueryOptions;
use crate::db::models::relation::RelationTripleQuery;
use crate::db::models::triple::TripleQueryResult;
use crate::store::TripleStore;
use crate::StoreError::{self, UnsupportedInputData};

impl TripleStore {
    pub fn print_sparql_result(&mut self, sparql: &str) -> Result<(), StoreError> {
        for triple in self.parse_sparql_query(sparql)? {
            match triple {
                TripleQueryResult::Relation {
                    subject,
                    predicate,
                    object,
                } => println!(
                    "{} {} {}",
                    self.get_short_form(subject)?,
                    self.get_short_form(predicate)?,
                    self.get_short_form(object)?
                ),
                TripleQueryResult::Property {
                    subject,
                    predicate,
                    literal_value,
                    ..
                } => println!(
                    "{} {} {}",
                    self.get_short_form(subject)?,
                    self.get_short_form(predicate)?,
                    literal_value
                ),
            }
            println!()
        }
        Ok(())
    }

    pub fn parse_sparql_query(
        &mut self,
        sparql: &str,
    ) -> Result<Vec<TripleQueryResult>, StoreError> {
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
    ) -> Result<Vec<TripleQueryResult>, StoreError> {
        match pattern {
            GraphPattern::Bgp { patterns } => {
                let mut all_results = Vec::new();
                for tp in patterns {
                    let results = self.execute_triple_pattern(tp, options)?;
                    all_results.extend(results);
                }
                Ok(all_results)
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

    fn execute_triple_pattern(
        &mut self,
        tp: TriplePattern,
        options: QueryOptions,
    ) -> Result<Vec<TripleQueryResult>, StoreError> {
        // Map SPARQL Subject (Variable or NamedNode) to Option<String>
        let s = match tp.subject {
            spargebra::term::TermPattern::NamedNode(named_node) => {
                Some(self.normalize_iri(named_node.as_str())?)
            }
            _ => None,
        };

        // Map SPARQL Predicate to Option<String>
        let p = match tp.predicate {
            spargebra::term::NamedNodePattern::NamedNode(named_node) => {
                Some(self.normalize_iri(named_node.as_str())?)
            }
            _ => None,
        };

        // Map SPARQL Object
        let res = match tp.object {
            spargebra::term::TermPattern::NamedNode(nn) => self.query_relation_triples_joined(
                RelationTripleQuery::with_values(s, p, Some(self.normalize_iri(nn.as_str())?)),
                options,
            ),
            spargebra::term::TermPattern::Literal(literal) => self.query_property_triples_joined(
                PropertyTripleQuery::with_values(
                    s,
                    p,
                    Some(literal.to_string()),
                    LiteralMatchMode::Exact,
                ),
                options,
            ),
            spargebra::term::TermPattern::Variable(_) => {
                let mut results = self.query_property_triples_joined(
                    PropertyTripleQuery::with_values(
                        s.clone(),
                        p.clone(),
                        None,
                        LiteralMatchMode::Exact,
                    ),
                    options,
                )?;
                results.extend(self.query_relation_triples_joined(
                    RelationTripleQuery::with_values(s, p, None),
                    options,
                )?);
                Ok(results)
            }
            _ => Err(UnsupportedInputData),
        }?;
        if let Some(lim) = options.limit {
            Ok(res[0..lim].to_vec())
        } else {
            Ok(res)
        }
    }
}
