use oxrdf::{NamedNode, NamedOrBlankNode, Term, Triple};
use pg_triple_store::query::solution::QueryResult;
use pg_triple_store::store::TripleStore;

use crate::common::init_store;

mod common;

#[test]
fn test_import_turtle_file() {
    let _ = env_logger::Builder::from_default_env()
        .filter(None, log::LevelFilter::Debug)
        .try_init();
    let store = TripleStore::new_from_env().expect("DB connection failed");
    store.reset_db().expect("reset failed");
    store.import_turtle_file("tests/test.ttl").unwrap()
}

#[test]
fn test_import_rdf_file() {
    let _ = env_logger::Builder::from_default_env()
        .filter(None, log::LevelFilter::Debug)
        .try_init();
    let store = TripleStore::new_from_env().expect("DB connection failed");
    store.reset_db().expect("reset failed");
    store.import_rdfxml_file("tests/test.rdf").unwrap()
}

#[test]
fn test_export_rdfxml_file() {
    let store = init_store();
    let sol = store
        .query(
            "
        CONSTRUCT {
            ?s ex:knows ex:alice .
        }
        WHERE {
            ex:alice ex:knows ?s .
        }",
        )
        .unwrap();
    let QueryResult::Graph(graph) = sol else {
        panic!("Construct Result should be of type graph")
    };
    println!("{}", store.export_as_rdfxml(&graph).unwrap());
}

#[test]
fn test_insert_triple() {
    let store = init_store();
    store
        .insert_triple(Triple {
            subject: NamedOrBlankNode::NamedNode(
                NamedNode::new("http://example.org/alice").unwrap(),
            ),
            predicate: NamedNode::new("http://example.org/worksAt").unwrap(),
            object: Term::NamedNode(NamedNode::new("http://example.org/orgA").unwrap()),
        })
        .unwrap();
}

#[test]
fn test_insert_triples() {
    let store = init_store();
    let triples = &[
        Triple {
            subject: NamedOrBlankNode::NamedNode(
                NamedNode::new("http://example.org/alice").unwrap(),
            ),
            predicate: NamedNode::new("http://example.org/worksAt").unwrap(),
            object: Term::NamedNode(NamedNode::new("http://example.org/orgA").unwrap()),
        },
        Triple {
            subject: NamedOrBlankNode::NamedNode(
                NamedNode::new("http://example.org/carol").unwrap(),
            ),
            predicate: NamedNode::new("http://example.org/knows").unwrap(),
            object: Term::NamedNode(NamedNode::new("http://example.org/alice").unwrap()),
        },
    ];
    store.insert_triples(triples).unwrap();
}

#[test]
fn test_describe_object() {
    let store = init_store();
    let id = store
        .get_id_from_iri(
            "http://example.org/alice",
            pg_triple_store::store::ElementType::Object,
        )
        .unwrap();
    println!("object id is {id}");
    let relations = store.describe_object_by_id(id).unwrap();
    assert_eq!(relations.len(), 6)
}

#[test]
fn test_describe_predicate() {
    let store = init_store();
    let id = store
        .get_id_from_iri(
            "http://example.org/knows",
            pg_triple_store::store::ElementType::Predicate,
        )
        .unwrap();
    println!("predicate id is {id}");
    let relations = store.describe_predicate_by_id(id).unwrap();
    assert_eq!(relations.len(), 5)
}

#[test]
fn test_relation_subject_list() {
    let store = init_store();
    let object_id = store
        .get_id_from_iri(
            "http://example.org/carol",
            pg_triple_store::store::ElementType::Object,
        )
        .unwrap();
    println!("object id is {object_id}");
    let predicate_id = store
        .get_id_from_iri(
            "http://example.org/knows",
            pg_triple_store::store::ElementType::Predicate,
        )
        .unwrap();
    println!("predicate id is {predicate_id}");
    let subjects = store
        .get_relation_subject_list(predicate_id, object_id)
        .unwrap();
    assert_eq!(subjects.len(), 2)
}

#[test]
fn test_shorten_iri() {
    let store = init_store();
    assert_eq!(
        store.get_short_iri("http://example.org/bob").unwrap(),
        "ex:bob"
    )
}

#[test]
fn test_add_prefix() {
    let store = init_store();
    store.add_prefix("http://local-test.org", "lt").unwrap()
}
