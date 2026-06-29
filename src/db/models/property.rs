use diesel::Selectable;
use diesel::deserialize::Queryable;
use diesel::prelude::Insertable;

use crate::db::models::triple::TriplePosition;
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

#[derive(Default, PartialEq, Eq)]
pub enum LiteralMatchMode {
    #[default]
    Exact,
    Contains,
}

pub struct PropertyTripleQuery {
    pub(crate) subject: TriplePosition,
    pub(crate) predicate: TriplePosition,
    pub(crate) literal_value: TriplePosition,
    pub(crate) literal_match_mode: LiteralMatchMode,
}

impl PropertyTripleQuery {
    pub fn with_values(
        subject: TriplePosition,
        predicate: TriplePosition,
        literal_value: TriplePosition,
        literal_match_mode: LiteralMatchMode,
    ) -> Self {
        Self {
            subject,
            predicate,
            literal_value,
            literal_match_mode,
        }
    }
}
