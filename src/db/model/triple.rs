use std::fmt::Display;

pub enum TripleQuery {
    PropertyTripleQuery {
        subject: TriplePosition,
        predicate: TriplePosition,
        object: TriplePosition,
    },
    RelationTripleQuery {
        subject: TriplePosition,
        predicate: TriplePosition,
        object_value: TriplePosition,
        object_match_mode: LiteralMatchMode,
    },
}

#[derive(Default, PartialEq, Eq)]
pub enum LiteralMatchMode {
    #[default]
    Exact,
    Contains,
}

#[derive(Clone)]
pub enum TriplePosition {
    Constant(String),
    Variable(String),
    Bound(String, Vec<Term>),
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
