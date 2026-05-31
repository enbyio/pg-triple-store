use std::collections::{HashMap, HashSet};

use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl};
use log::error;

use crate::db::models::object::{NewObject, Object};
use crate::store::TripleStore;
use crate::StoreError;

use crate::schema::objects::dsl::*;

impl TripleStore {
    pub fn upsert_object(&mut self, object: impl Into<NewObject>) -> Result<i64, StoreError> {
        let object_iri = object.into().iri;
        let mut conn = self.conn()?;
        if let Some(existing_id) = objects
            .filter(iri.eq(&object_iri))
            .select(id)
            .first::<i64>(&mut conn)
            .optional()?
        {
            return Ok(existing_id);
        }
        Ok(diesel::insert_into(objects)
            .values(iri.eq(&object_iri))
            .on_conflict(iri)
            .do_update()
            .set(iri.eq(&object_iri))
            .get_result::<Object>(&mut conn)?
            .id)
    }

    pub fn batch_upsert_objects(
        &mut self,
        object_iris: HashSet<NewObject>,
    ) -> Result<HashMap<String, i64>, StoreError> {
        let values: Vec<NewObject> = object_iris.into_iter().collect();
        let mut conn = self.conn()?;
        Ok(diesel::insert_into(objects)
            .values(&values)
            .on_conflict(iri)
            .do_update()
            .set(iri.eq(iri))
            .returning((iri, id))
            .get_results::<(String, i64)>(&mut conn)?
            .into_iter()
            .collect())
    }

    pub fn get_object_id(&mut self, object_iri: &str) -> Option<i64> {
        use crate::schema::objects::dsl::*;
        let mut conn = self.conn().ok()?;
        let res = objects
            .filter(iri.eq(object_iri))
            .select(id)
            .first::<i64>(&mut conn)
            .optional();
        if let Err(e) = &res {
            error!("Error trying to get object id {e:?}");
        }
        res.ok()?
    }
}
