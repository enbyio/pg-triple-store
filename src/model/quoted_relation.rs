use diesel::Selectable;
use diesel::deserialize::Queryable;
use diesel::prelude::Insertable;

use crate::schema::quoted_relations;

#[derive(Insertable, Queryable, Selectable, Debug, Hash)]
#[diesel(table_name=quoted_relations)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct QuotedRelation {
    id: i64,
    subject: i64,
    predicate: i64,
    object: i64,
}

impl QuotedRelation {
    pub fn new(id: i64, subject: i64, predicate: i64, object: i64) -> Self {
        Self {
            id,
            subject,
            predicate,
            object,
        }
    }
}
