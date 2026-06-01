use diesel::deserialize::Queryable;
use diesel::prelude::Insertable;
use diesel::Selectable;

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

pub struct RelationTripleQuery {
    pub(crate) subject: Option<String>,
    pub(crate) predicate: Option<String>,
    pub(crate) object: Option<String>,
}

impl RelationTripleQuery {
    pub fn with_values(
        subject: Option<String>,
        predicate: Option<String>,
        object: Option<String>,
    ) -> Self {
        Self {
            subject,
            predicate,
            object,
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

    pub fn object(mut self, object: String) -> Self {
        self.object = Some(object);
        self
    }
}
