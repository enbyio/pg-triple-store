use spargebra::algebra::{Expression, GraphPattern};

use crate::db::models::query::QueryResult;
use crate::store::TripleStore;
use crate::StoreError;

impl TripleStore {
    pub(crate) fn add_filter(
        pattern: GraphPattern,
        expr: Expression,
    ) -> Result<QueryResult, StoreError> {
        
        todo!()
    }
}
