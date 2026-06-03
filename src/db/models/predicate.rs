use diesel::deserialize::Queryable;
use diesel::prelude::Insertable;
use diesel::Selectable;

use crate::schema::predicates;

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name=predicates)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Predicate {
    pub id: i64,
    pub iri: String,
}

#[derive(Insertable, Hash, PartialEq, Eq)]
#[diesel(table_name=predicates)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewPredicate {
    pub(crate) iri: String,
}

impl NewPredicate {
    pub fn new(iri: String) -> Self {
        Self { iri }
    }
}

impl From<&str> for NewPredicate {
    fn from(value: &str) -> Self {
        Self::new(value.to_string())
    }
}
