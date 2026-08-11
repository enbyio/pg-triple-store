use diesel::Selectable;
use diesel::deserialize::Queryable;
use diesel::prelude::Insertable;

use crate::schema::properties;

#[derive(Insertable, Queryable, Selectable, Debug)]
#[diesel(table_name=properties)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Property {
    subject: i64,
    predicate: i64,
    literal_value: String,
    literal_type: Option<String>,
}

impl Property {
    pub fn new(
        subject: i64,
        predicate: i64,
        literal_value: String,
        literal_type: Option<String>,
    ) -> Self {
        Self {
            subject,
            predicate,
            literal_value,
            literal_type,
        }
    }
}
