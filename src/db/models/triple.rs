use oxrdf::NamedOrBlankNode;

use crate::StoreError;
use crate::db::models::property::{LiteralMatchMode, PropertyTripleQuery};
use crate::db::models::query::Term;
use crate::db::models::relation::RelationTripleQuery;

// pub enum TripleQuery {
//     Relation(RelationTripleQuery),
//     Property(PropertyTripleQuery),
// }

// impl TripleQuery {
//     pub fn relation(query: RelationTripleQuery) -> Self {
//         TripleQuery::Relation(query)
//     }

//     pub fn property(query: PropertyTripleQuery) -> Self {
//         TripleQuery::Property(query)
//     }
// }

#[derive(Clone)]
pub enum TriplePosition {
    Constant(String),
    Variable(String),
    Bound(String, Vec<Term>),
}

#[derive(Default)]
pub struct TripleQuery {
    subject: Option<TriplePosition>,
    predicate: Option<TriplePosition>,
    object: Option<TriplePosition>,
}

impl TripleQuery {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn subject(&mut self, subject: TriplePosition) {
        self.subject = Some(subject);
    }

    pub fn predicate(&mut self, predicate: TriplePosition) {
        self.predicate = Some(predicate);
    }

    pub fn object(&mut self, object: TriplePosition) {
        self.object = Some(object);
    }

    pub fn relation_query(&self) -> Result<RelationTripleQuery, StoreError> {
        if let Some(s) = self.subject.clone()
            && let Some(p) = self.predicate.clone()
            && let Some(o) = self.object.clone()
        {
            Ok(RelationTripleQuery::with_values(s, p, o))
        } else {
            Err(StoreError::DataError(
                "subject, predicate or object missing from triple query".to_string(),
            ))
        }
    }

    pub fn property_query(
        &self,
        match_type: LiteralMatchMode,
    ) -> Result<PropertyTripleQuery, StoreError> {
        if let Some(s) = self.subject.clone()
            && let Some(p) = self.predicate.clone()
            && let Some(o) = self.object.clone()
        {
            Ok(PropertyTripleQuery::with_values(s, p, o, match_type))
        } else {
            Err(StoreError::DataError(
                "subject, predicate or object missing from triple query".to_string(),
            ))
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
