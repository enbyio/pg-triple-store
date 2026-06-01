use diesel::deserialize::Queryable;
use diesel::prelude::Insertable;
use diesel::Selectable;

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

#[derive(Default)]
pub enum LiteralMatchMode {
    #[default]
    Exact,
    Contains,
}

#[derive(Default)]
pub struct PropertyTripleQuery {
    pub(crate) subject: Option<String>,
    pub(crate) predicate: Option<String>,
    pub(crate) literal_value: Option<String>,
    pub(crate) literal_match_mode: LiteralMatchMode,
}

impl PropertyTripleQuery {
    pub fn with_values(
        subject: Option<String>,
        predicate: Option<String>,
        literal_value: Option<String>,
        literal_match_mode: LiteralMatchMode,
    ) -> Self {
        Self {
            subject,
            predicate,
            literal_value,
            literal_match_mode,
        }
    }

    pub fn subject(mut self, subject: String) -> Self {
        self.subject = Some(subject);
        self
    }

    pub fn predicate(mut self, predicate: String) -> Self {
        self.predicate = Some(predicate);
        self
    }

    pub fn literal_value(mut self, literal_value: String) -> Self {
        self.literal_value = Some(literal_value);
        self
    }

    pub fn match_mode(mut self, mode: LiteralMatchMode) -> Self {
        self.literal_match_mode = mode;
        self
    }
}
