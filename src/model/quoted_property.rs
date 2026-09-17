use diesel::deserialize::Queryable;
use diesel::prelude::Insertable;
use diesel::Selectable;

use crate::schema::quoted_properties;

#[derive(Insertable, Queryable, Selectable, Debug, Hash)]
#[diesel(table_name=quoted_properties)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct QuotedProperty {
    id: i64,
    subject: i64,
    predicate: i64,
    literal_value: String,
    literal_type: Option<String>,
}

impl QuotedProperty {
    pub fn new(
        id: i64,
        subject: i64,
        predicate: i64,
        literal_value: String,
        literal_type: Option<String>,
    ) -> Self {
        Self {
            id,
            subject,
            predicate,
            literal_value,
            literal_type,
        }
    }
}
