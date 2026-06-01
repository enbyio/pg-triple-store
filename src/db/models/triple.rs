use crate::db::models::property::PropertyTripleQuery;
use crate::db::models::relation::RelationTripleQuery;

pub enum TripleQuery {
    Relation(RelationTripleQuery),
    Property(PropertyTripleQuery),
}

impl TripleQuery {
    pub fn relation(query: RelationTripleQuery) -> Self {
        TripleQuery::Relation(query)
    }

    pub fn property(query: PropertyTripleQuery) -> Self {
        TripleQuery::Property(query)
    }
}

#[derive(Debug)]
pub enum TripleQueryResult {
    Relation {
        subject: String,
        predicate: String,
        object: String,
    },
    Property {
        subject: String,
        predicate: String,
        literal_value: String,
        literal_type: Option<String>,
    },
}
