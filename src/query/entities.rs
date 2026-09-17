use diesel::RunQueryDsl;

use crate::error::StoreError;
use crate::model::entity::{EntityType, NewEntity};
use crate::store::{PgPooledConnection, TripleStore};

impl TripleStore {
    // TODO: maybe rename this?
    pub(crate) fn add_entity_with_session(
        &self,
        entity_type: EntityType,
        conn: &mut PgPooledConnection,
    ) -> Result<i64, StoreError> {
        use crate::schema::entities;
        Ok(diesel::insert_into(entities::table)
            .values(NewEntity {
                entity_type: entity_type as i16,
            })
            .returning(entities::id)
            .get_result(conn)?)
    }
}
