use crate::model::property::Property;
use crate::model::relation::Relation;
use std::fmt::Display;

// TODO: there has to be a better way for this
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum RelationOrProperty {
    Property(Property),
    Relation(Relation),
}

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
    BlankNode(String),
}

// note for later: it might be useful to differentiate between iri and bnode here but that needs changing in the test cases
impl Display for Term {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Term::Iri(iri) => write!(f, "{iri}"),
            Term::Literal { value, .. } => {
                write!(f, "{value}")
            }
            Term::BlankNode(name) => write!(f, "{name}"),
        }
    }
}
