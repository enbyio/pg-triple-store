use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl};

use crate::db::error::StoreError;
use crate::db::model::object::{NewObject, Object};
use crate::db::store::TripleStore;

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
}
