use pg_triple_store::store::TripleStore;

#[test]
fn setup_db() {
    let store = TripleStore::new_from_env();
    assert!(store.is_ok())
}

#[test]
fn test_migrate() {
    env_logger::Builder::from_default_env()
        .filter(None, log::LevelFilter::Info)
        .init();
    let mut store = TripleStore::new_from_env().unwrap();
    assert!(store.reset_db().is_ok());
    assert!(store
        .import_turtle_from_url("https://schema.org/version/latest/schemaorg-current-https.ttl")
        .is_ok());
    store
        .print_sparql_result(
            "SELECT ?s ?o
            WHERE {
                ?s rdf:type ?o
            }
            LIMIT 10",
        )
        .unwrap();
}
