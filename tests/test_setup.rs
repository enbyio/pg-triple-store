use pg_triple_store::db::query::solution::{QueryResult, SolutionSet};
use pg_triple_store::db::store::TripleStore;

fn init_store() -> TripleStore {
    let _ = env_logger::Builder::from_default_env()
        .filter(None, log::LevelFilter::Debug)
        .try_init();
    let mut store = TripleStore::new_from_env().expect("DB connection failed");
    store.reset_db().expect("reset failed");
    store
        .import_turtle_file("tests/test.ttl")
        .expect("turtle import failed");
    store
}

/// Execute a SELECT query and return the `SolutionSet`.
/// Panics with a descriptive message if the query errors or returns a non-Select result.
fn select(store: &mut TripleStore, sparql: &str) -> SolutionSet {
    match store.parse_sparql_query(sparql).expect(sparql) {
        QueryResult::Solutions(s) => s,
        other => panic!("expected Solutions, got: {other}"),
    }
}

/// Collect all values of one variable from a solution set.
fn values(sol: &SolutionSet, var: &str) -> Vec<String> {
    let mut out: Vec<String> = sol
        .rows
        .iter()
        .filter_map(|row| row.bindings.get(var))
        .map(|t| t.to_string())
        .collect();
    out.sort();
    out
}

/// Assert `expected` values (sorted) for one variable in a solution set.
fn assert_values(sol: &SolutionSet, var: &str, mut expected: Vec<&str>) {
    expected.sort();
    let got = values(sol, var);
    assert_eq!(
        got,
        expected.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
        "variable ?{var} mismatch\ngot:      {got:?}\nexpected: {expected:?}"
    );
}

fn assert_count(sol: &SolutionSet, n: usize) {
    assert_eq!(
        sol.rows.len(),
        n,
        "expected {n} rows, got {}\nrows: {:#?}",
        sol.rows.len(),
        sol.rows
    );
}

fn assert_empty(sol: &SolutionSet) {
    assert_count(sol, 0);
}

#[test]
fn test_cross_product_no_shared_vars() {
    let mut store = init_store();
    let sol = select(
        &mut store,
        "SELECT ?person ?org WHERE {
           ?person ex:knows ex:bob .
           ?org ex:partOf ex:consortium .
         }",
    );
    // alice knows bob; orgA and orgB both partOf consortium → 1 × 2 = 2 rows
    assert_count(&sol, 2);
}

#[test]
fn test_shared_var_intersection() {
    let mut store = init_store();
    let sol = select(
        &mut store,
        "SELECT ?person WHERE {
           ?person ex:knows ex:carol .
           ex:alice ex:knows ?person .
         }",
    );
    // Knows carol: alice, bob, carol(self?). alice knows: bob, carol.
    // Intersection of (?person knows carol) AND (alice knows ?person): bob
    assert_values(&sol, "person", vec!["http://example.org/bob"]);
}

#[test]
fn test_three_hop_chain() {
    let mut store = init_store();
    let sol = select(
        &mut store,
        "SELECT ?hop1 ?hop2 ?org WHERE {
           ex:alice ex:knows ?hop1 .
           ?hop1 ex:knows ?hop2 .
           ?hop2 ex:worksAt ?org .
         }",
    );
    // alice→bob→carol→orgB, alice→bob→dave(no worksAt), alice→carol→dave(no worksAt)
    // Only carol worksAt something → one row: hop1=bob, hop2=carol, org=orgB
    assert_count(&sol, 1);
    assert_values(&sol, "org", vec!["http://example.org/orgB"]);
}

// ── FILTER: equality ──────────────────────────────────────────────────────────

#[test]
fn test_filter_equality_string() {
    let mut store = init_store();
    let sol = select(
        &mut store,
        r#"SELECT ?p WHERE {
             ?p ex:name ?name .
             FILTER(?name = "Alice")
           }"#,
    );
    assert_values(&sol, "p", vec!["http://example.org/alice"]);
}

#[test]
fn test_filter_equality_or() {
    let mut store = init_store();
    let sol = select(
        &mut store,
        r#"SELECT ?p WHERE {
             ?p ex:name ?name .
             FILTER(?name = "Alice" || ?name = "Dave")
           }"#,
    );
    assert_values(
        &sol,
        "p",
        vec!["http://example.org/alice", "http://example.org/dave"],
    );
}

#[test]
fn test_filter_inequality() {
    let mut store = init_store();
    let sol = select(
        &mut store,
        r#"SELECT ?p WHERE {
             ?p ex:name ?name .
             FILTER(?name != "Alice")
           }"#,
    );
    // bob, carol, dave — not alice
    assert_count(&sol, 3);
    let names = values(&sol, "name");
    assert!(
        !names.iter().any(|n| n == "Alice"),
        "Alice should be excluded"
    );
}

#[test]
fn test_filter_equality_no_match() {
    let mut store = init_store();
    let sol = select(
        &mut store,
        r#"SELECT ?p WHERE {
             ?p ex:name ?name .
             FILTER(?name = "Zaphod")
           }"#,
    );
    assert_empty(&sol);
}

// ── FILTER: numeric comparisons ───────────────────────────────────────────────
// These require xsd:integer typed literals in the TTL (see updated test.ttl).

#[test]
fn test_filter_numeric_gt() {
    let mut store = init_store();
    // age > 25: only alice (30)
    let sol = select(
        &mut store,
        r#"SELECT ?p WHERE {
             ?p ex:age ?age .
             FILTER(?age > "25"^^xsd:integer)
           }"#,
    );
    println!("{:?}", sol);
    assert_values(&sol, "p", vec!["http://example.org/alice"]);
}

#[test]
fn test_filter_numeric_gte() {
    let mut store = init_store();
    // age >= 25: alice (30), bob (25), carol (25)
    let sol = select(
        &mut store,
        r#"SELECT ?p WHERE {
             ?p ex:age ?age .
             FILTER(?age >= "25"^^xsd:integer)
           }"#,
    );
    log::debug!("{:?}", sol.rows.len());
    assert_count(&sol, 3);
}

#[test]
fn test_filter_numeric_lt() {
    let mut store = init_store();
    // age < 30: bob and carol (both 25)
    let sol = select(
        &mut store,
        r#"SELECT ?p WHERE {
             ?p ex:age ?age .
             FILTER(?age < "30"^^xsd:integer)
           }"#,
    );
    assert_count(&sol, 2);
    assert_values(
        &sol,
        "p",
        vec!["http://example.org/bob", "http://example.org/carol"],
    );
}

#[test]
fn test_filter_numeric_eq_integer() {
    let mut store = init_store();
    let sol = select(
        &mut store,
        r#"SELECT ?p WHERE {
             ?p ex:age ?age .
             FILTER(?age = "25"^^xsd:integer)
           }"#,
    );
    assert_count(&sol, 2);
}

// ── FILTER: logical NOT ───────────────────────────────────────────────────────

#[test]
fn test_filter_not() {
    let mut store = init_store();
    let sol = select(
        &mut store,
        r#"SELECT ?p WHERE {
             ?p ex:name ?name .
             FILTER(!(?name = "Alice"))
           }"#,
    );
    assert_count(&sol, 3);
    let names = values(&sol, "name");
    assert!(!names.contains(&"Alice".to_string()));
}

// ── FILTER: logical AND ───────────────────────────────────────────────────────

#[test]
fn test_filter_and() {
    let mut store = init_store();
    // age = 25 AND knows dave
    let sol = select(
        &mut store,
        r#"SELECT ?p WHERE {
             ?p ex:age ?age .
             ?p ex:knows ex:dave .
             FILTER(?age = "25"^^xsd:integer)
           }"#,
    );
    // bob (25, knows dave) and carol (25, knows dave)
    assert_count(&sol, 2);
}

#[test]
fn test_filter_compound_and_or() {
    let mut store = init_store();
    // (name = "Alice" || name = "Bob") && age >= 25
    let sol = select(
        &mut store,
        r#"SELECT ?p WHERE {
             ?p ex:name ?name .
             ?p ex:age ?age .
             FILTER((?name = "Alice" || ?name = "Bob") && ?age >= "25"^^xsd:integer)
           }"#,
    );
    assert_count(&sol, 2);
    assert_values(&sol, "name", vec!["Alice", "Bob"]);
}

// ── FILTER: BOUND ─────────────────────────────────────────────────────────────
// Dave has no ex:age → he won't appear in a BGP that matches ex:age.
// These tests use the pattern where the variable may or may not be bound
// via an OPTIONAL-style approach. Since OPTIONAL isn't implemented yet,
// we test BOUND by checking that the BGP itself excludes unbound rows,
// and confirm via NOT BOUND that dave has no name-linked age.

#[test]
fn test_filter_bound_age_exists() {
    let mut store = init_store();
    // Everyone who has an age — dave must NOT appear
    let sol = select(
        &mut store,
        r#"SELECT ?p WHERE {
             ?p ex:age ?age .
             FILTER(BOUND(?age))
           }"#,
    );
    assert_count(&sol, 3); // alice, bob, carol
    let names: Vec<String> = sol
        .rows
        .iter()
        .map(|r| r.bindings["p"].to_string())
        .collect();
    assert!(!names.iter().any(|n| n.contains("dave")));
}

// ── FILTER: IN ────────────────────────────────────────────────────────────────

#[test]
fn test_filter_in_list() {
    let mut store = init_store();
    let sol = select(
        &mut store,
        r#"SELECT ?p WHERE {
             ?p ex:name ?name .
             FILTER(?name IN ("Alice", "Carol"))
           }"#,
    );
    assert_count(&sol, 2);
    assert_values(&sol, "name", vec!["Alice", "Carol"]);
}

#[test]
fn test_filter_not_in_list() {
    let mut store = init_store();
    let sol = select(
        &mut store,
        r#"SELECT ?p WHERE {
             ?p ex:name ?name .
             FILTER(?name NOT IN ("Alice", "Carol"))
           }"#,
    );
    assert_count(&sol, 2);
    assert_values(&sol, "name", vec!["Bob", "Dave"]);
}

// ── FILTER: string functions ──────────────────────────────────────────────────

#[test]
fn test_filter_contains() {
    let mut store = init_store();
    let sol = select(
        &mut store,
        r#"SELECT ?p WHERE {
             ?p ex:name ?name .
             FILTER(CONTAINS(?name, "li"))
           }"#,
    );
    // "Alice" contains "li"
    assert_values(&sol, "p", vec!["http://example.org/alice"]);
}

#[test]
fn test_filter_strstarts() {
    let mut store = init_store();
    let sol = select(
        &mut store,
        r#"SELECT ?p WHERE {
             ?p ex:name ?name .
             FILTER(STRSTARTS(?name, "A"))
           }"#,
    );
    assert_values(&sol, "p", vec!["http://example.org/alice"]);
}

#[test]
fn test_filter_strends() {
    let mut store = init_store();
    let sol = select(
        &mut store,
        r#"SELECT ?p WHERE {
             ?p ex:name ?name .
             FILTER(STRENDS(?name, "b"))
           }"#,
    );
    // "Bob" ends with "b"
    assert_values(&sol, "p", vec!["http://example.org/bob"]);
}

#[test]
fn test_filter_strlen_gt() {
    let mut store = init_store();
    let sol = select(
        &mut store,
        r#"SELECT ?p WHERE {
             ?p ex:name ?name .
             FILTER(STRLEN(?name) > "4"^^xsd:integer)
           }"#,
    );
    // "Alice" (5), "Carol" (5) — "Bob" (3), "Dave" (4) excluded
    assert_count(&sol, 2);
    assert_values(&sol, "name", vec!["Alice", "Carol"]);
}

#[test]
fn test_filter_lcase() {
    let mut store = init_store();
    let sol = select(
        &mut store,
        r#"SELECT ?p WHERE {
             ?p ex:name ?name .
             FILTER(LCASE(?name) = "alice")
           }"#,
    );
    assert_values(&sol, "p", vec!["http://example.org/alice"]);
}

#[test]
fn test_filter_ucase() {
    let mut store = init_store();
    let sol = select(
        &mut store,
        r#"SELECT ?p WHERE {
             ?p ex:name ?name .
             FILTER(UCASE(?name) = "BOB")
           }"#,
    );
    assert_values(&sol, "p", vec!["http://example.org/bob"]);
}

// ── FILTER: REGEX ─────────────────────────────────────────────────────────────
// Requires `regex = "1"` in Cargo.toml.

#[test]
fn test_filter_regex_basic() {
    let mut store = init_store();
    let sol = select(
        &mut store,
        r#"SELECT ?p WHERE {
             ?p ex:name ?name .
             FILTER(REGEX(?name, "^[AB]"))
           }"#,
    );
    // Alice, Bob start with A or B
    assert_count(&sol, 2);
    assert_values(&sol, "name", vec!["Alice", "Bob"]);
}

#[test]
fn test_filter_regex_case_insensitive() {
    let mut store = init_store();
    let sol = select(
        &mut store,
        r#"SELECT ?p WHERE {
             ?p ex:name ?name .
             FILTER(REGEX(?name, "alice", "i"))
           }"#,
    );
    assert_values(&sol, "name", vec!["Alice"]);
}

#[test]
fn test_filter_regex_no_match() {
    let mut store = init_store();
    let sol = select(
        &mut store,
        r#"SELECT ?p WHERE {
             ?p ex:name ?name .
             FILTER(REGEX(?name, "^Z"))
           }"#,
    );
    assert_empty(&sol);
}

// ── FILTER: IRI-side filtering ────────────────────────────────────────────────
// Filters on the subject IRI itself (the SQL Eq pushdown path).

#[test]
fn test_filter_on_iri_variable() {
    let mut store = init_store();
    let sol = select(
        &mut store,
        r#"SELECT ?name WHERE {
             ?p ex:name ?name .
             FILTER(?p = ex:alice)
           }"#,
    );
    assert_values(&sol, "name", vec!["Alice"]);
}

#[test]
fn test_filter_isiri() {
    let mut store = init_store();
    // All objects of ex:knows are IRIs (not literals)
    let sol = select(
        &mut store,
        r#"SELECT ?friend WHERE {
             ex:alice ex:knows ?friend .
             FILTER(isIRI(?friend))
           }"#,
    );
    assert_count(&sol, 2); // bob, carol
}

#[test]
fn test_filter_isliteral() {
    let mut store = init_store();
    let sol = select(
        &mut store,
        r#"SELECT ?val WHERE {
             ex:alice ?p ?val .
             FILTER(isLiteral(?val))
           }"#,
    );
    // "Alice", "30"^^xsd:integer, "9.5"^^xsd:decimal → 3 literal values
    assert_count(&sol, 3);
}

// ── FILTER: filter on joined results ─────────────────────────────────────────

#[test]
fn test_filter_after_join() {
    let mut store = init_store();
    let sol = select(
        &mut store,
        r#"SELECT ?person ?friend WHERE {
             ?person ex:knows ?friend .
             ?friend ex:age ?age .
             FILTER(?age = "25"^^xsd:integer)
           }"#,
    );
    // Knows someone with age 25:
    //   alice→bob(25), alice→carol(25)
    //   bob→carol(25), bob→dave(no age — excluded by BGP)
    //   carol→dave(no age — excluded)
    assert_count(&sol, 3);
}

#[test]
fn test_filter_age_range_with_name() {
    let mut store = init_store();
    let sol = select(
        &mut store,
        r#"SELECT ?p ?name WHERE {
             ?p ex:name ?name .
             ?p ex:age ?age .
             FILTER(?age >= "25"^^xsd:integer && ?age < "30"^^xsd:integer)
           }"#,
    );
    // age in [25, 30): bob (25), carol (25) — alice (30) excluded
    assert_count(&sol, 2);
    assert_values(&sol, "name", vec!["Bob", "Carol"]);
}

#[test]
fn test_filter_org_member_young() {
    let mut store = init_store();
    let sol = select(
        &mut store,
        r#"SELECT ?p ?org WHERE {
             ?p ex:worksAt ?org .
             ?p ex:age ?age .
             FILTER(?age < "30"^^xsd:integer)
           }"#,
    );
    // bob(25)→orgA, carol(25)→orgB
    assert_count(&sol, 2);
}

// ── FILTER: arithmetic in filter expression ───────────────────────────────────

#[test]
fn test_filter_arithmetic() {
    let mut store = init_store();
    // score + 1.0 > 9.0: alice (9.5+1=10.5), carol (8.0+1=9.0 — NOT > 9.0)
    // so only alice's score satisfies score > 8.0, i.e. score + 0 > 8.0
    // Let's use: score * 2 > 17 → only alice (9.5*2=19 > 17)
    let sol = select(
        &mut store,
        r#"SELECT ?p WHERE {
             ?p ex:score ?s .
             FILTER(?s * "2"^^xsd:decimal > "17"^^xsd:decimal)
           }"#,
    );
    assert_values(&sol, "p", vec!["http://example.org/alice"]);
}

// ── FILTER: IF expression ─────────────────────────────────────────────────────

#[test]
fn test_filter_if_expression() {
    let mut store = init_store();
    // IF(?name = "Alice", true, false) → keep only alice
    let sol = select(
        &mut store,
        r#"SELECT ?p WHERE {
             ?p ex:name ?name .
             FILTER(IF(?name = "Alice", true, false))
           }"#,
    );
    assert_values(&sol, "p", vec!["http://example.org/alice"]);
}

// ── FILTER: COALESCE ──────────────────────────────────────────────────────────

#[test]
fn test_filter_coalesce_fallback() {
    let mut store = init_store();
    // COALESCE(?missing, "Alice") = "Alice" → all rows where ?name = result
    // Here every row has a name, so we compare ?name to COALESCE of a constant
    let sol = select(
        &mut store,
        r#"SELECT ?p WHERE {
             ?p ex:name ?name .
             FILTER(COALESCE(?name, "Fallback") = "Alice")
           }"#,
    );
    assert_values(&sol, "p", vec!["http://example.org/alice"]);
}

// ── FILTER: complex real-world-like query ─────────────────────────────────────

#[test]
fn test_filter_complex_social_graph() {
    let mut store = init_store();
    // "Find all pairs (person, colleague) who work at the same org,
    //  where person's name starts with 'A' and colleague's age <= 25"
    let sol = select(
        &mut store,
        r#"SELECT ?person ?colleague WHERE {
             ?person    ex:worksAt ?org .
             ?colleague ex:worksAt ?org .
             ?person    ex:name    ?pname .
             ?colleague ex:age     ?cage .
             FILTER(STRSTARTS(?pname, "A") && ?cage <= "25"^^xsd:integer && ?person != ?colleague)
           }"#,
    );
    // alice works at orgA; bob also at orgA (age 25)
    // alice's name starts with A, bob's age = 25
    assert_count(&sol, 1);
    assert_values(&sol, "colleague", vec!["http://example.org/bob"]);
}

// ── edge cases ────────────────────────────────────────────────────────────────

#[test]
fn test_filter_on_empty_bgp_result() {
    let mut store = init_store();
    // The BGP itself returns nothing (no one knows ex:nobody),
    // so the filter should also return nothing.
    let sol = select(
        &mut store,
        r#"SELECT ?p WHERE {
             ?p ex:knows ex:nobody .
             FILTER(?p = ex:alice)
           }"#,
    );
    assert_empty(&sol);
}

#[test]
fn test_filter_always_false() {
    let mut store = init_store();
    let sol = select(
        &mut store,
        r#"SELECT ?p WHERE {
             ?p ex:name ?name .
             FILTER(false)
           }"#,
    );
    assert_empty(&sol);
}

#[test]
fn test_filter_always_true() {
    let mut store = init_store();
    let sol = select(
        &mut store,
        r#"SELECT ?p WHERE {
             ?p ex:name ?name .
             FILTER(true)
           }"#,
    );
    // All four people have a name
    assert_count(&sol, 4);
}

#[test]
fn test_filter_same_term() {
    let mut store = init_store();
    // sameTerm is stricter than = but for plain string literals they agree
    let sol = select(
        &mut store,
        r#"SELECT ?p WHERE {
             ?p ex:name ?name .
             FILTER(sameTerm(?name, "Alice"))
           }"#,
    );
    assert_values(&sol, "p", vec!["http://example.org/alice"]);
}

// ── regression: filter must not affect un-filtered queries ────────────────────

#[test]
fn test_no_filter_returns_all_names() {
    let mut store = init_store();
    let sol = select(
        &mut store,
        "SELECT ?p ?name WHERE {
           ?p ex:name ?name .
         }",
    );
    assert_count(&sol, 4);
    assert_values(&sol, "name", vec!["Alice", "Bob", "Carol", "Dave"]);
}
