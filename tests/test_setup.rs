use pg_triple_store::store::TripleStore;

#[test]
fn setup_db() {
    let store = TripleStore::new_from_env();
    assert!(store.is_ok())
}

#[test]
fn load_test_turtle_file() {
    env_logger::Builder::from_default_env()
        .filter(None, log::LevelFilter::Info)
        .init();
    let mut store = TripleStore::new_from_env().unwrap();
    assert!(store.reset_db().is_ok());
    assert!(store.import_turtle_file("tests/test_data/test.ttl").is_ok());
}

#[test]
fn test_triple_parsing() {
    env_logger::Builder::from_default_env()
        .filter(None, log::LevelFilter::Info)
        .init();
    let mut store = TripleStore::new_from_env().unwrap();
    assert!(store.reset_db().is_ok());
    assert!(store.import_turtle_file("tests/test_data/test.ttl").is_ok());

    // Case 1:
    println!(
        "{}",
        store
            .parse_sparql_query(
                "SELECT ?friend ?name WHERE {
      ex:alice ex:knows ?friend .
      ?friend ex:name ?name .
    }"
            )
            .unwrap()
    );

    println!(
        "{}",
        store
            .parse_sparql_query(
                "SELECT ?person ?org WHERE {
      ?person ex:knows ex:bob .
      ?org ex:partOf ex:consortium .
    }"
            )
            .unwrap()
    );

    println!(
        "{}",
        store
            .parse_sparql_query(
                "SELECT ?person WHERE {
      ?person ex:knows ex:carol .
      ex:alice ex:knows ?person .
    }"
            )
            .unwrap()
    );

    println!(
        "{}",
        store
            .parse_sparql_query(
                "SELECT ?name WHERE {
      ex:alice ex:knows ex:bob .
      ex:alice ex:name ?name .
    }"
            )
            .unwrap()
    );

    println!(
        "{}",
        store
            .parse_sparql_query(
                "SELECT ?hop1 ?hop2 ?org WHERE {
      ex:alice ex:knows ?hop1 .
      ?hop1 ex:knows ?hop2 .
      ?hop2 ex:worksAt ?org .
    }"
            )
            .unwrap()
    );

    println!(
        "{}",
        store
            .parse_sparql_query(
                "SELECT ?person ?age WHERE {
      ?person ex:worksAt ex:orgA .
      ?person ex:age ?age .
    }"
            )
            .unwrap()
    );

    println!(
        "{}",
        store
            .parse_sparql_query(
                "SELECT ?a ?b ?age WHERE {
      ?a ex:age ?age .
      ?b ex:age ?age .
    }"
            )
            .unwrap()
    );

    println!(
        "{}",
        store
            .parse_sparql_query(
                "SELECT DISTINCT ?d ?c WHERE {
        ?a ex:knows ?b .
        ?a ex:name ?d .
        ?b ex:name ?c .
        }"
            )
            .unwrap()
    )
}
