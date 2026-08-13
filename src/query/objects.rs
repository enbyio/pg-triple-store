use std::collections::{HashMap, HashSet};

use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl};

use crate::error::StoreError;
use crate::model::object::{NewObject, Object};
use crate::store::TripleStore;

use crate::schema::objects::dsl::*;

impl TripleStore {
    /** Checks if an object with a given iri exists and if not inserts it. ID of the object is returned regardless
     */
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
            .on_conflict_do_nothing()
            .get_result::<Object>(&mut conn)?
            .id)
    }

    /** checks if an object exists and returns either said objects id or none, if the object doesn't exist
     */
    pub fn get_object_id(&mut self, object_iri: impl Into<String>) -> Option<i64> {
        let object_iri = object_iri.into();
        let mut conn = self
            .conn()
            .inspect_err(|e| log::error!("Failed to establish connection: {:?}", e))
            .ok()?;
        objects
            .filter(iri.eq(object_iri))
            .select(id)
            .first::<i64>(&mut conn)
            .optional()
            .inspect_err(|e| log::error!("Failed to query DB for object: {:?}", e))
            .ok()?
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
}
