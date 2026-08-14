use diesel::Selectable;
use diesel::deserialize::Queryable;
use diesel::prelude::Insertable;

use crate::schema::objects;

/// An object with iri and id
/// "id" is the primary key in the table
#[derive(Queryable, Selectable, Debug, Hash)]
#[diesel(table_name=objects)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Object {
    pub id: i64,
    pub iri: String,
}

/// The insertable type for object, takes only the iri and autofills the id
#[derive(Insertable, PartialEq, Eq, Hash)]
#[diesel(table_name=objects)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewObject {
    pub(crate) iri: String,
}

impl NewObject {
    pub fn new(iri: String) -> Self {
        Self { iri }
    }
}

impl From<&str> for NewObject {
    fn from(value: &str) -> Self {
        Self::new(value.to_string())
    }
}
