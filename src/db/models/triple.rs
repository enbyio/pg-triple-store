use oxrdf::NamedOrBlankNode;

use crate::db::models::property::PropertyTripleQuery;
use crate::db::models::relation::RelationTripleQuery;
use crate::StoreError;

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

#[derive(Debug, Clone)]
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

impl TripleQueryResult {
    pub fn relation(values: (String, String, String)) -> Self {
        Self::Relation {
            subject: values.0,
            predicate: values.1,
            object: values.2,
        }
    }

    pub fn property(values: (String, String, String)) -> Self {
        Self::Property {
            subject: values.0,
            predicate: values.1,
            literal_value: values.2,
            literal_type: None,
        }
    }
}

pub trait AsIri {
    fn as_iri(&self) -> Result<&str, StoreError>;
}

impl AsIri for NamedOrBlankNode {
    fn as_iri(&self) -> Result<&str, StoreError> {
        match self {
            NamedOrBlankNode::NamedNode(named_node) => Ok(named_node.as_str()),
            NamedOrBlankNode::BlankNode(_) => Err(StoreError::UnsupportedInputData),
        }
    }
}
