use diesel::Selectable;
use diesel::deserialize::Queryable;
use diesel::prelude::Insertable;

use crate::schema::relations;

#[derive(Insertable, Queryable, Selectable, Debug, Default)]
#[diesel(table_name=relations)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Relation {
    subject: i64,
    predicate: i64,
    object: i64,
}

impl Relation {
    pub fn new(subject: i64, predicate: i64, object: i64) -> Self {
        Self {
            subject,
            predicate,
            object,
        }
    }
}
