use std::collections::HashSet;

use spargebra::term::{NamedNodePattern, TermPattern, TriplePattern};

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
