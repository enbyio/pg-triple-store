#[derive(Clone, Copy, Debug, Default)]
pub struct QueryOptions {
    pub limit: Option<usize>,
    pub offset: usize,
}
use std::collections::BTreeMap;
use std::fmt::Display;

use oxrdf::Triple;

/// One row of a SPARQL solution.
#[derive(Debug, Clone, PartialEq)]
pub struct Solution {
    pub(crate) bindings: BTreeMap<String, Term>, // var name -> term (BTreeMap = stable ordering)
}

impl Solution {
    pub fn new(map: BTreeMap<String, Term>) -> Self {
        Self { bindings: map }
    }
}

impl Display for Solution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s: String = self
            .bindings
            .iter()
            .map(|(k, v)| format!("{}: {}", k, v))
            .collect::<Vec<String>>()
            .join(", ");
        write!(f, "{s}")
    }
}

/// Your own term type — decouples you from spargebra/oxrdf at the result boundary.
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
            Term::Literal { value, datatype } => {
                if datatype != "Unknown" {
                    write!(f, "{value}")
                } else {
                    write!(f, "{value}@{datatype}")
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct SolutionSet {
    pub vars: Vec<String>,
    pub rows: Vec<Solution>,
}

#[derive(Debug)]
pub enum QueryResult {
    Solutions(SolutionSet),
    Boolean(bool),
    Graph(Vec<Triple>),
}

impl Display for QueryResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QueryResult::Solutions(solution_set) => {
                let sol = solution_set
                    .rows
                    .iter()
                    .map(|a| format!("({a})"))
                    .collect::<Vec<String>>()
                    .join(", ");
                write!(f, "{:?}: [{}]", solution_set.vars, sol)
            }
            QueryResult::Boolean(val) => write!(f, "{val}"),
            QueryResult::Graph(graph) => {
                write!(f, "Graph output is not supported yet: {:?}", graph)
            }
        }
    }
}

#[derive(Default)]
pub struct SolutionBuilder {
    pub subject_name: Option<String>,
    pub predicate_name: Option<String>,
    pub object_name: Option<String>,
}

impl SolutionBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn subject(&mut self, value: String) {
        self.subject_name = Some(value);
    }

    pub fn predicate(&mut self, value: String) {
        self.predicate_name = Some(value);
    }

    pub fn object(&mut self, value: String) {
        self.object_name = Some(value);
    }

    pub fn get_rel_solutions(&self, results: Vec<(String, String, String)>) -> Vec<Solution> {
        results
            .into_iter()
            .map(|res| {
                let mut map: BTreeMap<String, Term> = BTreeMap::new();
                if let Some(s) = &self.subject_name {
                    map.insert(s.clone(), Term::Iri(res.0));
                }
                if let Some(p) = &self.predicate_name {
                    map.insert(p.clone(), Term::Iri(res.1));
                }
                if let Some(o) = &self.object_name {
                    map.insert(o.clone(), Term::Iri(res.2));
                }
                Solution::new(map)
            })
            .collect()
    }

    pub fn get_prop_solutions(&self, results: Vec<(String, String, String)>) -> Vec<Solution> {
        results
            .into_iter()
            .map(|res| {
                let mut map: BTreeMap<String, Term> = BTreeMap::new();
                if let Some(s) = &self.subject_name {
                    map.insert(s.clone(), Term::Iri(res.0));
                }
                if let Some(p) = &self.predicate_name {
                    map.insert(p.clone(), Term::Iri(res.1));
                }
                if let Some(o) = &self.object_name {
                    map.insert(
                        o.clone(),
                        Term::Literal {
                            value: res.2,
                            datatype: "Unknown".to_string(),
                        },
                    );
                }
                Solution::new(map)
            })
            .collect()
    }
}
