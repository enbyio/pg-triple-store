use std::collections::BTreeMap;
use std::fmt::Display;

use oxrdf::Triple;

use crate::model::triple::{Term, VarKey};

#[derive(Debug, Clone, PartialEq)]
pub struct Solution {
    pub(crate) bindings: BTreeMap<VarKey, Term>, // var name -> term (BTreeMap = stable ordering)
}

impl Solution {
    pub(crate) fn new(map: BTreeMap<VarKey, Term>) -> Self {
        Self { bindings: map }
    }

    /// Look up a binding by SPARQL variable name (e.g. "p" for `?p`).
    /// Not sure if this is useful in actual usage but this is used for testing purposes
    /// Blank-node bindings are not reachable through this API.
    pub fn get(&self, name: &str) -> Option<&Term> {
        self.bindings.get(&VarKey::Named(name.to_string()))
    }
}

impl Display for Solution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s: String = self
            .bindings
            .iter()
            .map(|(k, v)| format!("{:?}: {}", k, v))
            .collect::<Vec<String>>()
            .join(", ");
        write!(f, "{s}")
    }
}

#[derive(Default)]
pub(crate) struct SolutionBuilder {
    pub subject_name: Option<VarKey>,
    pub predicate_name: Option<VarKey>,
    pub object_name: Option<VarKey>,
}

impl SolutionBuilder {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn subject(&mut self, value: VarKey) {
        self.subject_name = Some(value);
    }

    pub(crate) fn predicate(&mut self, value: VarKey) {
        self.predicate_name = Some(value);
    }

    pub(crate) fn object(&mut self, value: VarKey) {
        self.object_name = Some(value);
    }

    pub(crate) fn get_rel_solutions(
        &self,
        results: Vec<(String, String, String)>,
    ) -> Vec<Solution> {
        results
            .into_iter()
            .map(|res| {
                let mut map: BTreeMap<VarKey, Term> = BTreeMap::new();
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

    pub(crate) fn get_prop_solutions_typed(
        &self,
        results: Vec<(String, String, String, Option<String>)>,
    ) -> Vec<Solution> {
        use std::collections::BTreeMap;
        results
            .into_iter()
            .map(|res| {
                let mut map: BTreeMap<VarKey, Term> = BTreeMap::new();
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
                            datatype: res.3.unwrap_or_else(|| {
                                "http://www.w3.org/2001/XMLSchema#string".to_string()
                            }),
                        },
                    );
                }
                Solution::new(map)
            })
            .collect()
    }
}

#[derive(Debug, Default)]
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
                    .map(|row| {
                        solution_set
                            .vars
                            .iter()
                            .map(|var| {
                                format!(
                                    "{:?}: {}",
                                    var,
                                    row.bindings.get(&VarKey::Named(var.clone())).unwrap()
                                )
                            })
                            .collect::<Vec<String>>()
                            .join(", ")
                    })
                    .collect::<Vec<String>>()
                    .join("), (");
                write!(f, "{:?}: [({})]", solution_set.vars, sol)
            }
            QueryResult::Boolean(val) => write!(f, "{val}"),
            QueryResult::Graph(graph) => {
                write!(f, "Graph output is not supported yet: {:?}", graph)
            }
        }
    }
}
