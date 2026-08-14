use oxrdf::Triple;

use crate::error::StoreError;
use crate::query::solution::QueryResult;
use crate::store::TripleStore;

impl TripleStore {
    pub fn query(&mut self, sparql: impl Into<String>) -> Result<QueryResult, StoreError> {
        let query = sparql.into();
        self.parse_sparql_query(&query)
    }

    pub fn insert_triple(&mut self, triple: Triple) -> Result<(), StoreError> {
        self.upsert_triple(triple)
    }

    pub fn insert_triples(&mut self, triples: &[Triple]) -> Result<(), StoreError> {
        self.batch_upsert_triples(triples)
    }

    pub fn add_prefix(
        &mut self,
        namespace: impl Into<String>,
        prefix: impl Into<String>,
    ) -> Result<(), StoreError> {
        self.upsert_prefix(prefix.into(), namespace.into())
    }
}
