#![allow(dead_code)]
use pg_triple_store::query::solution::{QueryResult, SolutionSet};
use pg_triple_store::store::TripleStore;

pub fn init_store() -> TripleStore {
    let _ = env_logger::Builder::from_default_env()
        .filter(None, log::LevelFilter::Debug)
        .try_init();
    let store = TripleStore::new_from_env().expect("DB connection failed");
    store.reset_db().expect("reset failed");
    store
        .import_turtle_file("tests/test.ttl")
        .expect("turtle import failed");
    store
}

/// Execute a SELECT query and return the `SolutionSet`.
/// Panics with a descriptive message if the query errors or returns a non-Select result.
pub fn select(store: &TripleStore, sparql: &str) -> SolutionSet {
    match store.query(sparql).expect(sparql) {
        QueryResult::Solutions(s) => s,
        other => panic!("expected Solutions, got: {other}"),
    }
}

/// Collect all values of one variable from a solution set.
pub fn values(sol: &SolutionSet, var: &str) -> Vec<String> {
    let mut out: Vec<String> = sol
        .rows
        .iter()
        .filter_map(|row| row.get(var))
        .map(|t| t.to_string())
        .collect();
    out.sort();
    out
}

/// Assert `expected` values (sorted) for one variable in a solution set.
pub fn assert_values(sol: &SolutionSet, var: &str, mut expected: Vec<&str>) {
    expected.sort();
    let got = values(sol, var);
    assert_eq!(
        got,
        expected.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
        "variable ?{var} mismatch\ngot:      {got:?}\nexpected: {expected:?}"
    );
}

pub fn assert_count(sol: &SolutionSet, n: usize) {
    assert_eq!(
        sol.rows.len(),
        n,
        "expected {n} rows, got {}\nrows: {:#?}",
        sol.rows.len(),
        sol.rows
    );
}

pub fn assert_empty(sol: &SolutionSet) {
    assert_count(sol, 0);
}
