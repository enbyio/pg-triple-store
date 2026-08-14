use oxrdf::Triple;

use crate::error::StoreError;
use crate::query::solution::QueryResult;
use crate::store::{ElementType, TripleStore};

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

    pub fn get_short_iri(&mut self, curie_iri: impl Into<String>) -> Result<String, StoreError> {
        self.shorten_iri(curie_iri.into())
    }

    pub fn get_id_from_iri(&mut self, iri: impl Into<String>, id_type: ElementType) -> Option<i64> {
        match id_type {
            ElementType::Object => self.get_object_id(iri.into()),
            ElementType::Predicate => self.get_predicate_id(iri.into()),
        }
    }
}
