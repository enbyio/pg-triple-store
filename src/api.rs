use oxrdf::Triple;

use crate::error::StoreError;
use crate::query::solution::QueryResult;
use crate::store::{ElementType, TripleStore};

impl TripleStore {
    /// Function to run sparql queries in the triple store.
    pub fn query(&self, sparql: impl Into<String>) -> Result<QueryResult, StoreError> {
        let query = sparql.into();
        self.parse_sparql_query(&query)
    }

    /// Function to insert a single triple. Takes anything that can implements Into<oxrdf::Triple>.
    pub fn insert_triple(&self, triple: impl Into<Triple>) -> Result<(), StoreError> {
        self.upsert_triple(triple.into())
    }

    /// Function to insert a slice of type oxrdf::Triple. \
    /// Use this for larger imports (turtle also resolves to this).
    pub fn insert_triples(&self, triples: &[Triple]) -> Result<(), StoreError> {
        self.batch_upsert_triples(triples)
    }

    /// Function to add a single prefix, if needed.
    pub fn add_prefix(
        &self,
        namespace: impl Into<String>,
        prefix: impl Into<String>,
    ) -> Result<(), StoreError> {
        self.upsert_prefix(prefix.into(), namespace.into())
    }

    /// Shorten a long form (CURIE) iri into a short form. \
    /// **Warning:** Current version is a bit flimsy so giving this a short form might result in an error.
    pub fn get_short_iri(&self, curie_iri: impl Into<String>) -> Result<String, StoreError> {
        self.shorten_iri(curie_iri.into())
    }

    /// Returns the id for a given iri (object or predicate). \
    /// Used for the describe id function.
    pub fn get_id_from_iri(&self, iri: impl Into<String>, id_type: ElementType) -> Option<i64> {
        match id_type {
            ElementType::Object => self.get_object_id(iri.into()),
            ElementType::Predicate => self.get_predicate_id(iri.into()),
        }
    }

    /// Describe an object (aka return all relations and properties of said object).
    /// Returns a list of all triples with this id either as the subject or object (in the relations)
    pub fn describe_object_by_id(&self, id: i64) -> Result<Vec<Triple>, StoreError> {
        self.describe_object_id(id)
    }

    /// Describe a predicate (aka return all relations and properties containing said predicate)
    /// Returns a list of all triples with this id as the predicate (analog function to describe_object_by_id)
    pub fn describe_predicate_by_id(&self, id: i64) -> Result<Vec<Triple>, StoreError> {
        self.describe_predicate_id(id)
    }

    /// get a list of subjects with a certain object and predicate for relations
    /// the name might need to change, but this implements a pattern that is for example useful for a content repository (get all elements of a certain type)
    pub fn get_relation_subject_list(
        &self,
        predicate: i64,
        object: i64,
    ) -> Result<Vec<(String, i64)>, StoreError> {
        self.get_list_of_relation_subjects(predicate, object)
    }
}
