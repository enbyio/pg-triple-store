use pg_triple_store::store::TripleStore;

#[test]
fn setup_db() {
    let store = TripleStore::new_from_env();
    assert!(store.is_ok())
}
