use diesel::deserialize::Queryable;
use diesel::prelude::Insertable;
use diesel::Selectable;

use crate::schema::prefixes;

#[derive(Insertable, Queryable, Selectable, Debug)]
#[diesel(table_name=prefixes)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Prefix {
    namespace: String,
    prefix: String,
}

impl Prefix {
    pub fn new(namespace: String, prefix: String) -> Self {
        Self { namespace, prefix }
    }
}
