use std::fmt::Display;

use oxrdf::NamedOrBlankNode;

use crate::error::StoreError;

#[derive(Default, PartialEq, Eq)]
pub enum LiteralMatchMode {
    #[default]
    Exact,
    #[allow(dead_code)]
    Contains,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub(crate) enum VarKey {
    Named(String),
    Blank(String),
}

impl VarKey {
    pub(crate) fn is_named(&self) -> bool {
        matches!(self, VarKey::Named(_))
    }
    pub(crate) fn into_name(self) -> String {
        match self {
            VarKey::Named(s) | VarKey::Blank(s) => s,
        }
    }
}

#[derive(Clone)]
pub(crate) enum TriplePosition {
    Constant(String),
    Variable(VarKey),
    Bound(VarKey, Vec<Term>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Term {
    Iri(String),
    Literal { value: String, datatype: String },
    // BlankNode(String), later
}

impl Display for Term {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Term::Iri(iri) => write!(f, "{iri}"),
            Term::Literal { value, .. } => {
                // if datatype.eq("Unknown") {
                //     write!(f, "{value}")
                // } else {
                //     write!(f, "{value}@{datatype}")
                // }
                write!(f, "{value}")
            }
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
            NamedOrBlankNode::BlankNode(_) => Err(StoreError::data_error(
                "Blank Node is not supported as input data here",
            )),
        }
    }
}
