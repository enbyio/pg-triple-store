use spargebra::algebra::GraphPattern;
use spargebra::term::TriplePattern;
use spargebra::{Query, SparqlParser};

use crate::db::models::property::{LiteralMatchMode, PropertyTripleQuery};
use crate::db::models::relation::RelationTripleQuery;
use crate::db::models::triple::TripleQueryResult;
use crate::store::TripleStore;
use crate::StoreError::{self, UnsupportedInputData};

impl TripleStore {
    pub fn parse_sparql_query(
        &mut self,
        sparql: &str,
    ) -> Result<Vec<TripleQueryResult>, StoreError> {
        let parser = SparqlParser::new();
        let query = parser.parse_query(sparql)?;
        match query {
            Query::Select { pattern, .. } => self.execute_pattern(pattern),
            Query::Construct { .. } => Err(UnsupportedInputData),
            Query::Describe { .. } => Err(UnsupportedInputData),
            Query::Ask { .. } => Err(UnsupportedInputData),
        }
    }

    fn execute_pattern(
        &mut self,
        pattern: GraphPattern,
    ) -> Result<Vec<TripleQueryResult>, StoreError> {
        match pattern {
            GraphPattern::Bgp { patterns } => {
                let mut all_results = Vec::new();
                for tp in patterns {
                    let results = self.execute_triple_pattern(tp)?;
                    all_results.extend(results);
                }
                Ok(all_results)
            }
            _ => Err(UnsupportedInputData),
        }
    }

    fn execute_triple_pattern(
        &mut self,
        tp: TriplePattern,
    ) -> Result<Vec<TripleQueryResult>, StoreError> {
        // Map SPARQL Subject (Variable or NamedNode) to Option<String>
        let s = match tp.subject {
            spargebra::term::TermPattern::NamedNode(named_node) => Some(named_node.to_string()),
            _ => None,
        };

        // Map SPARQL Predicate to Option<String>
        let p = match tp.predicate {
            spargebra::term::NamedNodePattern::NamedNode(named_node) => {
                Some(named_node.to_string())
            }
            _ => None,
        };

        // Map SPARQL Object
        match tp.object {
            spargebra::term::TermPattern::NamedNode(nn) => self.query_relation_triples_joined(
                RelationTripleQuery::with_values(s, p, Some(nn.to_string())),
            ),
            spargebra::term::TermPattern::Literal(literal) => {
                self.query_property_triples_joined(PropertyTripleQuery::with_values(
                    s,
                    p,
                    Some(literal.to_string()),
                    LiteralMatchMode::Exact,
                ))
            }
            spargebra::term::TermPattern::Variable(_) => {
                let mut results =
                    self.query_property_triples_joined(PropertyTripleQuery::with_values(
                        s.clone(),
                        p.clone(),
                        None,
                        LiteralMatchMode::Exact,
                    ))?;
                results.extend(
                    self.query_relation_triples_joined(RelationTripleQuery::with_values(
                        s, p, None,
                    ))?,
                );
                Ok(results)
            }
            _ => Err(UnsupportedInputData),
        }
    }
}
