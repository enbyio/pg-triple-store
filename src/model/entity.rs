use diesel::prelude::Insertable;

#[repr(i16)]
pub(crate) enum EntityType {
    Object = 1,
    QuotedRelation = 2,
    QuotedProperty = 3,
}

#[derive(Insertable)]
#[diesel(table_name=crate::schema::entities)]
pub struct NewEntity {
    pub entity_type: i16,
}
