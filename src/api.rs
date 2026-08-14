use oxrdf::Triple;

use crate::error::StoreError;
use crate::query::solution::QueryResult;
use crate::store::{ElementType, TripleStore};

impl TripleStore {
    /// Function to run sparql queries in the triple store.
    pub fn query(&mut self, sparql: impl Into<String>) -> Result<QueryResult, StoreError> {
        let query = sparql.into();
        self.parse_sparql_query(&query)
    }

    /// Function to insert a single triple. Takes anything that can implements Into<oxrdf::Triple>.
    pub fn insert_triple(&mut self, triple: impl Into<Triple>) -> Result<(), StoreError> {
        self.upsert_triple(triple.into())
    }

    /// Function to insert a slice of type oxrdf::Triple. \
    /// Use this for larger imports (turtle also resolves to this).
    pub fn insert_triples(&mut self, triples: &[Triple]) -> Result<(), StoreError> {
        self.batch_upsert_triples(triples)
    }

    /// Function to add a single prefix, if needed.
    pub fn add_prefix(
        &mut self,
        namespace: impl Into<String>,
        prefix: impl Into<String>,
    ) -> Result<(), StoreError> {
        self.upsert_prefix(prefix.into(), namespace.into())
    }

    /// Shorten a long form (CURIE) iri into a short form. \
    /// **Warning:** Current version is a bit flimsy so giving this a short form might result in an error.
    pub fn get_short_iri(&mut self, curie_iri: impl Into<String>) -> Result<String, StoreError> {
        self.shorten_iri(curie_iri.into())
    }

    /// Returns the id for a given iri (object or predicate). \
    /// *Note:* currently not much useful, but will be used for the describe function added soon.
    pub fn get_id_from_iri(&mut self, iri: impl Into<String>, id_type: ElementType) -> Option<i64> {
        match id_type {
            ElementType::Object => self.get_object_id(iri.into()),
            ElementType::Predicate => self.get_predicate_id(iri.into()),
        }
    }
}
