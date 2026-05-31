use diesel::{ExpressionMethods, RunQueryDsl};
use log::{debug, info};
use oxrdf::Literal;

use crate::db::models::property::Property;
use crate::store::TripleStore;
use crate::StoreError;

impl TripleStore {
    pub fn create_property_triple(
        &mut self,
        subject_id: i64,
        predicate_id: i64,
        object: Literal,
    ) -> Result<(), StoreError> {
        use crate::schema::properties::dsl::*;
        let mut conn = self.conn()?;
        let object_value = object.to_string();
        let object_type = object.datatype().to_string();
        let affected_rows = diesel::insert_into(properties)
            .values((
                subject.eq(subject_id),
                predicate.eq(predicate_id),
                literal_value.eq(&object_value),
                literal_type.eq(&object_type),
            ))
            .on_conflict((subject, predicate, literal_value))
            .do_nothing()
            .execute(&mut conn)?;

        if affected_rows == 0 {
            debug!(
                "Duplicate property triple skipped for subject={}, predicate={}",
                subject_id, predicate_id
            );
        } else {
            debug!(
                "Rows affected from property triple insert: {}",
                affected_rows
            );
        }
        Ok(())
    }

    pub fn batch_create_property_triple(&mut self, triples: &[Property]) -> Result<(), StoreError> {
        use crate::schema::properties::dsl::*;
        let mut conn = self.conn()?;
        let affected_rows = diesel::insert_into(properties)
            .values(triples)
            .on_conflict_do_nothing()
            .execute(&mut conn)?;
        info!(
            "BATCH INSERT PROPERTIES for {} elements affected {} rows",
            triples.len(),
            affected_rows
        );
        Ok(())
    }
}
