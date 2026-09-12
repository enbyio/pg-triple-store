use diesel::deserialize::{FromSqlRow, Queryable};
use diesel::expression::AsExpression;
use diesel::pg::Pg;
use diesel::prelude::Insertable;
use diesel::sql_types::Text;
use diesel::Selectable;

use crate::schema::objects;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, AsExpression, FromSqlRow)]
#[diesel(sql_type = Text)]
pub enum ObjectKind {
    Iri,
    Blank,
}

impl diesel::serialize::ToSql<Text, Pg> for ObjectKind {
    fn to_sql<'b>(
        &'b self,
        out: &mut diesel::serialize::Output<'b, '_, Pg>,
    ) -> diesel::serialize::Result {
        let s = match self {
            ObjectKind::Iri => "iri",
            ObjectKind::Blank => "blank",
        };
        <str as diesel::serialize::ToSql<Text, Pg>>::to_sql(s, out)
    }
}

impl diesel::deserialize::FromSql<Text, Pg> for ObjectKind {
    fn from_sql(bytes: diesel::pg::PgValue<'_>) -> diesel::deserialize::Result<Self> {
        let s = <String as diesel::deserialize::FromSql<Text, Pg>>::from_sql(bytes)?;
        match s.as_str() {
            "iri" => Ok(ObjectKind::Iri),
            "blank" => Ok(ObjectKind::Blank),
            other => Err(format!("Unrecognized term_kind: {other}").into()),
        }
    }
}

/// An object with iri and id
/// "id" is the primary key in the table
#[derive(Queryable, Selectable, Debug, Hash)]
#[diesel(table_name=objects)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Object {
    pub id: i64,
    pub kind: ObjectKind,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ObjectKey {
    pub(crate) value: String,
    pub(crate) kind: ObjectKind,
}

impl ObjectKey {
    pub fn new_iri(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            kind: ObjectKind::Iri,
        }
    }
    pub fn new_blank(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            kind: ObjectKind::Blank,
        }
    }

    pub fn hash(&self) -> i64 {
        use std::hash::{Hash, Hasher};
        let mut h = std::collections::hash_map::DefaultHasher::new();
        self.kind.hash(&mut h);
        self.value.hash(&mut h);
        h.finish() as i64
    }
}

// replaces the old `impl From<&str> for NewObject`
impl From<&str> for ObjectKey {
    fn from(value: &str) -> Self {
        Self::new_iri(value)
    }
}

/// The insertable type for object, takes only the iri and autofills the id
#[derive(Insertable, PartialEq, Eq, Hash)]
#[diesel(table_name=objects)]
#[diesel(check_for_backend(Pg))]
pub struct NewObject {
    pub(crate) id: i64,
    pub(crate) value: String,
    pub(crate) kind: ObjectKind,
}

impl NewObject {
    pub(crate) fn from_key(key: ObjectKey, id: i64) -> Self {
        Self {
            id,
            value: key.value,
            kind: key.kind,
        }
    }
}

#[derive(Insertable, PartialEq, Eq, Hash)]
#[diesel(table_name=objects)]
#[diesel(check_for_backend(Pg))]
pub struct NewBlankNode {
    pub(crate) value: String,
    pub(crate) kind: ObjectKind,
}

impl NewBlankNode {
    pub fn new(iri: String) -> Self {
        Self {
            value: iri,
            kind: ObjectKind::Blank,
        }
    }
}
