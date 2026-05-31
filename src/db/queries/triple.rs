use std::collections::HashSet;

use log::{error, info};
use oxrdf::{Term, Triple};

use crate::StoreError;
use crate::db::models::object::NewObject;
use crate::db::models::predicate::NewPredicate;
use crate::db::models::property::Property;
use crate::db::models::relation::Relation;
use crate::store::TripleStore;

impl TripleStore {
    pub fn upsert_triple(&mut self, triple: Triple) -> Result<(), StoreError> {
        let subject_id = self.upsert_object(triple.subject.to_string())?;
        let predicate_id = self.upsert_predicate(triple.predicate.to_string())?;
        match triple.object {
            oxrdf::Term::NamedNode(named_node) => {
                let object_id = self.upsert_object(named_node.to_string())?;
                self.create_relation_triple(subject_id, predicate_id, object_id)
            }
            oxrdf::Term::BlankNode(_) => {
                error!("Blank nodes are not yet supported");
                Err(StoreError::UnsupportedInputData)
            }
            oxrdf::Term::Literal(literal) => {
                self.create_property_triple(subject_id, predicate_id, literal)
            }
            oxrdf::Term::Triple(_) => {
                error!("Recursive Triples are not yet supported");
                Err(StoreError::UnsupportedInputData)
            }
        }
    }

    pub fn batch_upsert_triples(&mut self, triples: &[Triple]) -> Result<(), StoreError> {
        let mut predicates: HashSet<NewPredicate> = HashSet::new();
        let mut objects: HashSet<NewObject> = HashSet::new();
        for triple in triples {
            objects.insert(triple.subject.to_string().into());
            predicates.insert(triple.predicate.to_string().into());
            if let Term::NamedNode(val) = &triple.object {
                objects.insert(val.to_string().into());
            }
        }
        info!("Found {} unique objects", objects.len());
        info!("Found {} unique predicates", predicates.len());
        let predicate_ids = self.batch_upsert_predicates(predicates)?;
        let object_ids = self.batch_upsert_objects(objects)?;
        let mut relations: Vec<Relation> = Vec::new();
        let mut properties: Vec<Property> = Vec::new();
        for triple in triples {
            if let Some(&subject) = object_ids.get(&triple.subject.to_string())
                && let Some(&predicate) = predicate_ids.get(&triple.predicate.to_string())
            {
                match &triple.object {
                    Term::NamedNode(named_node) => {
                        if let Some(&object) = object_ids.get(&named_node.to_string()) {
                            relations.push(Relation::new(subject, predicate, object));
                        } else {
                            error!("could not find id for the iri {}", named_node)
                        }
                    }
                    Term::Literal(literal) => properties.push(Property::new(subject, predicate, literal.to_string(), None)),
                    _ => error!("Blank Node and Triples in Triples are not supported yet"),
                }
            } else {
                error!("missing either subject or predicate id");
            }
        }
        self.batch_create_relation_triple(&relations)?;
        self.batch_create_property_triple(&properties)?;
        Ok(())
    }
}
