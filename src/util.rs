use std::collections::HashSet;

use spargebra::term::{NamedNodePattern, TermPattern, TriplePattern};

use crate::db::models::query::Solution;

pub(crate) fn triple_pattern_vars(pattern: &TriplePattern) -> HashSet<String> {
    let mut set: HashSet<String> = HashSet::new();
    if let TermPattern::Variable(var) = &pattern.subject {
        set.insert(var.to_string());
    }
    if let NamedNodePattern::Variable(var) = &pattern.predicate {
        set.insert(var.to_string());
    }
    if let TermPattern::Variable(var) = &pattern.object {
        set.insert(var.to_string());
    }
    set
}

pub(crate) fn merge(left: &Solution, right: &Solution) -> Solution {
    let mut bindings = left.bindings.clone();
    bindings.extend(right.bindings.clone());
    Solution { bindings }
}
