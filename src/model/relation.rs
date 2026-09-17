use diesel::Selectable;
use diesel::deserialize::Queryable;
use diesel::prelude::Insertable;

use crate::model::quoted_relation::QuotedRelation;
use crate::schema::relations;

#[derive(Insertable, Queryable, Selectable, Debug, Default, Clone, Copy, Hash, PartialEq, Eq)]
#[diesel(table_name=relations)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Relation {
    pub(crate) subject: i64,
    pub(crate) predicate: i64,
    pub(crate) object: i64,
}

impl Relation {
    pub fn new(subject: i64, predicate: i64, object: i64) -> Self {
        Self {
            subject,
            predicate,
            object,
        }
    }

    pub fn quote(&self, id: i64) -> QuotedRelation {
        QuotedRelation::new(id, self.subject, self.predicate, self.object)
    }

    pub fn hash(&self) -> i64 {
        use std::hash::{Hash, Hasher};
        let mut h = std::collections::hash_map::DefaultHasher::new();
        self.subject.hash(&mut h);
        self.predicate.hash(&mut h);
        self.object.hash(&mut h);
        h.finish() as i64
    }
}
