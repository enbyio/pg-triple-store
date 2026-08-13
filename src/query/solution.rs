use std::collections::BTreeMap;
use std::fmt::Display;

use oxrdf::Triple;

use crate::model::triple::Term;

#[derive(Debug, Clone, PartialEq)]
pub struct Solution {
    pub bindings: BTreeMap<String, Term>, // var name -> term (BTreeMap = stable ordering)
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

    pub(crate) fn get_rel_solutions(
        &self,
        results: Vec<(String, String, String)>,
    ) -> Vec<Solution> {
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

    pub(crate) fn get_prop_solutions_typed(
        &self,
        results: Vec<(String, String, String, Option<String>)>,
    ) -> Vec<Solution> {
        use std::collections::BTreeMap;
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
                            .map(|var| format!("{}: {}", var, row.bindings.get(var).unwrap()))
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
