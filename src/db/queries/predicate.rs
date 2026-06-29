use std::collections::{HashMap, HashSet};

use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl};
use log::error;

use crate::StoreError;
use crate::db::models::predicate::{NewPredicate, Predicate};
use crate::store::TripleStore;

use crate::schema::predicates::dsl::*;

impl TripleStore {
    pub fn upsert_predicate(
        &mut self,
        predicate_iri: impl Into<NewPredicate>,
    ) -> Result<i64, StoreError> {
        let mut conn = self.conn()?;
        let predicate_iri = predicate_iri.into().iri;
        if let Some(existing_id) = predicates
            .filter(iri.eq(&predicate_iri))
            .select(id)
            .first::<i64>(&mut conn)
            .optional()?
        {
            return Ok(existing_id);
        }
        Ok(diesel::insert_into(predicates)
            .values(iri.eq(&predicate_iri))
            .on_conflict(iri)
            .do_update()
            .set(iri.eq(&predicate_iri))
            .get_result::<Predicate>(&mut conn)?
            .id)
    }

    pub fn batch_upsert_predicates(
        &mut self,
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

    pub fn get_predicate_id(&mut self, predicate_iri: &str) -> Option<i64> {
        let mut conn = self.conn().ok()?;

        let res = predicates
            .filter(iri.eq(predicate_iri))
            .select(id)
            .first::<i64>(&mut conn)
            .optional();
        if let Err(e) = &res {
            error!("Error trying to get object id {e:?}");
        }
        res.ok()?
    }
}
