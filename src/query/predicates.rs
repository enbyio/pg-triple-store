use std::collections::{HashMap, HashSet};

use crate::error::StoreError;
use crate::model::predicate::{NewPredicate, Predicate};
use crate::store::TripleStore;
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl};

use crate::schema::predicates::dsl::*;

impl TripleStore {
    /// Checks if a predicate with a given iri exists and if not inserts it. ID of the predicate is returned either way
    pub(crate) fn upsert_predicate(
        &self,
        predicate_iri: impl Into<NewPredicate>,
    ) -> Result<i64, StoreError> {
        let mut conn = self.conn()?;
        let predicate_iri = predicate_iri.into();
        if let Some(existing_id) = predicates
            .filter(iri.eq(&predicate_iri.iri))
            .select(id)
            .first::<i64>(&mut conn)
            .optional()?
        {
            return Ok(existing_id);
        }
        Ok(diesel::insert_into(predicates)
            .values(&predicate_iri)
            .on_conflict_do_nothing()
            .get_result::<Predicate>(&mut conn)?
            .id)
    }

    /// checks if a predicate exists and returns either said objects id or none, if the object doesn't exist
    pub(crate) fn get_predicate_id(&self, predicate_iri: impl Into<String>) -> Option<i64> {
        let predicate_iri = predicate_iri.into();
        let mut conn = self
            .conn()
            .inspect_err(|e| log::error!("Failed to establish connection: {:?}", e))
            .ok()?;
        predicates
            .filter(iri.eq(predicate_iri))
            .select(id)
            .first::<i64>(&mut conn)
            .optional()
            .inspect_err(|e| log::error!("Failed to query DB for object: {:?}", e))
            .ok()?
    }

    pub(crate) fn batch_upsert_predicates(
        &self,
        object_iris: HashSet<NewPredicate>,
    ) -> Result<HashMap<String, i64>, StoreError> {
        let mut conn = self.conn()?;
        let values: Vec<NewPredicate> = object_iris.into_iter().collect();

        Ok(diesel::insert_into(predicates)
            .values(&values)
            .on_conflict(iri)
            .do_update()
            .set(iri.eq(iri))
            .returning((iri, id))
            .get_results::<(String, i64)>(&mut conn)?
            .into_iter()
            .collect())
    }
}
