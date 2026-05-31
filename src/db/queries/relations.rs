use diesel::{ExpressionMethods, RunQueryDsl};
use log::{debug, info};

use crate::db::models::relation::Relation;
use crate::store::TripleStore;
use crate::StoreError;

impl TripleStore {
    pub fn create_relation_triple(
        &mut self,
        subject_id: i64,
        predicate_id: i64,
        object_id: i64,
    ) -> Result<(), StoreError> {
        use crate::schema::relations::dsl::*;
        let mut conn = self.conn()?;
        let affected_rows = diesel::insert_into(relations)
            .values((
                subject.eq(subject_id),
                predicate.eq(predicate_id),
                object.eq(object_id),
            ))
            .on_conflict((subject, predicate, object))
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

    pub fn batch_create_relation_triple(&mut self, triples: &[Relation]) -> Result<(), StoreError> {
        use crate::schema::relations::dsl::*;
        let mut conn = self.conn()?;
        let affected_rows = diesel::insert_into(relations)
            .values(triples)
            .on_conflict_do_nothing()
            .execute(&mut conn)?;
        info!(
            "BATCH INSERT RELATIONS for {} elements affected {} rows",
            triples.len(),
            affected_rows
        );
        Ok(())
    }
}
