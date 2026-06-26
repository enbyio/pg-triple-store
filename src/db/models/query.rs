#[derive(Clone, Copy, Debug, Default)]
pub struct QueryOptions {
    pub limit: Option<usize>,
    pub offset: usize,
}
use std::collections::BTreeMap;

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

/// Your own term type — decouples you from spargebra/oxrdf at the result boundary.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Term {
    Iri(String),
    Literal { value: String, datatype: String },
    // BlankNode(String), later
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
