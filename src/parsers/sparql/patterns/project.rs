use oxrdf::Variable;
use spargebra::algebra::GraphPattern;

use crate::StoreError;
use crate::db::models::query::{QueryOptions, QueryResult, SolutionSet};
use crate::store::TripleStore;

impl TripleStore {
    pub(crate) fn project_pattern(
        &mut self,
        pattern: GraphPattern,
        variables: Vec<Variable>,
    ) -> Result<QueryResult, StoreError> {
        let result = self.execute_pattern(pattern, QueryOptions::default())?;
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
            QueryResult::Boolean(_) => Err(StoreError::DataError(
                "Project is invalid for boolean results".to_string(),
            )),
            QueryResult::Graph(_) => Err(StoreError::DataError(
                "Project is invalid for Graph results".to_string(),
            )),
        }
    }
}
