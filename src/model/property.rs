use diesel::deserialize::Queryable;
use diesel::prelude::Insertable;
use diesel::Selectable;

use crate::model::quoted_property::QuotedProperty;
use crate::schema::properties;

#[derive(Insertable, Queryable, Selectable, Debug)]
#[diesel(table_name=properties)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Property {
    pub(crate) subject: i64,
    pub(crate) predicate: i64,
    pub(crate) literal_value: String,
    pub(crate) literal_type: Option<String>,
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

    pub fn quote(&self, id: i64) -> QuotedProperty {
        QuotedProperty::new(
            id,
            self.subject,
            self.predicate,
            self.literal_value.clone(),
            self.literal_type.clone(),
        )
    }

    pub fn hash(&self) -> i64 {
        use std::hash::{Hash, Hasher};
        let mut h = std::collections::hash_map::DefaultHasher::new();
        self.subject.hash(&mut h);
        self.predicate.hash(&mut h);
        self.literal_type.hash(&mut h);
        self.literal_value.hash(&mut h);
        h.finish() as i64
    }
}
