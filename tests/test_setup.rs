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
        .filter(None, log::LevelFilter::Debug)
        .init();
    let mut store = TripleStore::new_from_env().unwrap();
    assert!(store.reset_db().is_ok());
    assert!(store.import_turtle_file("tests/test_data/test.ttl").is_ok());

    // Case 1:
    println!(
        "{:?}",
        store
            .parse_sparql_query(
                "SELECT ?friend ?name WHERE {
      ex:alice ex:knows ?friend .
      ?friend ex:name ?name .
    }",
            )
            .unwrap()
    );
}

// #[test]
// fn test_migrate() {
//     env_logger::Builder::from_default_env()
//         .filter(None, log::LevelFilter::Info)
//         .init();
//     let mut store = TripleStore::new_from_env().unwrap();
//     // assert!(store.reset_db().is_ok());
//     // assert!(store
//     //     .import_turtle_from_url("https://schema.org/version/latest/schemaorg-current-https.ttl")
//     //     .is_ok());
//     store
//         .print_sparql_result(
//             "SELECT ?s ?o
//             WHERE {
//                 ?s rdfs:label ?o
//             }",
//         )
//         .unwrap();
// }
