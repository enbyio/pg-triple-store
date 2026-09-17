use std::collections::{HashMap, HashSet};

use diesel::sql_types::BigInt;
use diesel::{
    BoolExpressionMethods, Connection, ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl,
    sql_query,
};

use crate::error::StoreError;
use crate::model::object::{NewObject, ObjectKey};
use crate::store::TripleStore;

impl TripleStore {
    /// Checks if an object with a given iri exists and if not inserts it. ID of the object is returned regardless
    pub(crate) fn upsert_object(&self, key: impl Into<ObjectKey>) -> Result<i64, StoreError> {
        use crate::schema::objects::dsl::*;
        let key = key.into();
        let mut conn = self.conn()?;
        conn.transaction(|conn| {
            let lock_key = key.hash();
            sql_query("SELECT pg_advisory_xact_lock($1)")
                .bind::<BigInt, _>(lock_key)
                .execute(conn)?;
            if let Some(existing) = objects
                .filter(kind.eq(key.kind).and(value.eq(&key.value)))
                .select(id)
                .first::<i64>(conn)
                .optional()?
            {
                return Ok(existing);
            }

            let new_id: i64 =
                self.add_entity_with_session(crate::model::entity::EntityType::Object, conn)?;

            diesel::insert_into(objects)
                .values(NewObject::from_key(key, new_id))
                .execute(conn)?;

            Ok(new_id)
        })
    }

    /// checks if an object exists and returns either said objects id or none, if the object doesn't exist
    pub(crate) fn get_object_id(&self, object_iri: impl Into<String>) -> Option<i64> {
        use crate::schema::objects::dsl::*;
        let object_iri = object_iri.into();
        let mut conn = self
            .conn()
            .inspect_err(|e| log::error!("Failed to establish connection: {:?}", e))
            .ok()?;
        objects
            .filter(value.eq(object_iri))
            .select(id)
            .first::<i64>(&mut conn)
            .optional()
            .inspect_err(|e| log::error!("Failed to query DB for object: {:?}", e))
            .ok()?
    }

    pub(crate) fn batch_upsert_objects(
        &self,
        keys: HashSet<ObjectKey>,
    ) -> Result<HashMap<ObjectKey, i64>, StoreError> {
        use crate::schema::objects::dsl::*;
        let mut conn = self.conn()?;
        conn.transaction(|conn| {
            let mut sorted_keys: Vec<ObjectKey> = keys.into_iter().collect();
            sorted_keys.sort_by_key(|k| k.hash());
            for k in &sorted_keys {
                sql_query("SELECT pg_advisory_xact_lock($1)")
                    .bind::<BigInt, _>(k.hash())
                    .execute(conn)?;
            }

            // find which already exist
            let mut result: HashMap<ObjectKey, i64> = HashMap::new();
            let mut missing: Vec<ObjectKey> = Vec::new();
            for k in sorted_keys {
                if let Some(existing_id) = objects
                    .filter(kind.eq(k.kind).and(value.eq(&k.value)))
                    .select(id)
                    .first::<i64>(conn)
                    .optional()?
                {
                    result.insert(k, existing_id);
                } else {
                    missing.push(k.clone());
                }
            }
            // mint entity ids for the missing ones, then insert objects
            for k in missing {
                let new_id: i64 =
                    self.add_entity_with_session(crate::model::entity::EntityType::Object, conn)?;
                diesel::insert_into(objects)
                    .values(NewObject::from_key(k.clone(), new_id))
                    .execute(conn)?;
                result.insert(k, new_id);
            }

            Ok(result)
        })
    }
}
