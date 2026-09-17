use std::collections::{HashMap, HashSet, VecDeque};

use diesel::{alias, ExpressionMethods, JoinOnDsl, QueryDsl, RunQueryDsl, TextExpressionMethods};
use ordermap::OrderSet;
use oxrdf::Triple;

use crate::model;

use crate::error::StoreError;
use crate::model::object::ObjectKey;
use crate::model::predicate::NewPredicate;
use crate::model::property::Property;
use crate::model::relation::Relation;
use crate::model::triple::{LiteralMatchMode, RelationOrProperty, TriplePosition};
use crate::query::solution::{Solution, SolutionBuilder};
use crate::store::{PgPooledConnection, TripleStore};

impl TripleStore {
    pub(crate) fn create_property_triple(&self, property: Property) -> Result<(), StoreError> {
        use crate::schema::properties::dsl::*;
        let mut conn = self.conn()?;
        let affected_rows = diesel::insert_into(properties)
            .values(&property)
            .on_conflict_do_nothing()
            .execute(&mut conn)?;
        if affected_rows == 0 {
            log::debug!("Skipped duplicate property triple: {:?}", property)
        } else {
            log::debug!("New property triple imported")
        }
        Ok(())
    }

    pub(crate) fn batch_create_property_triples(
        &self,
        props: &[Property],
    ) -> Result<(), StoreError> {
        use crate::schema::properties::dsl::*;
        let mut conn = self.conn()?;
        let affected_rows = diesel::insert_into(properties)
            .values(props)
            .on_conflict_do_nothing()
            .execute(&mut conn)?;
        log::debug!(
            "BATCH INSERT PROPERTIES for {} elements affected {} rows",
            props.len(),
            affected_rows
        );
        Ok(())
    }

    pub(crate) fn create_relation_triple(&self, relation: Relation) -> Result<(), StoreError> {
        use crate::schema::relations::dsl::*;
        let mut conn = self.conn()?;
        let affected_rows = diesel::insert_into(relations)
            .values(&relation)
            .on_conflict_do_nothing()
            .execute(&mut conn)?;
        if affected_rows == 0 {
            log::debug!("Skipped duplicate relation triple: {:?}", relation)
        } else {
            log::debug!("New relation triple imported")
        }
        Ok(())
    }

    pub(crate) fn batch_create_relation_triples(
        &self,
        rels: &[Relation],
    ) -> Result<(), StoreError> {
        use crate::schema::relations::dsl::*;
        let mut conn = self.conn()?;
        let affected_rows = diesel::insert_into(relations)
            .values(rels)
            .on_conflict_do_nothing()
            .execute(&mut conn)?;
        log::debug!(
            "BATCH INSERT RELATIONS for {} elements affected {} rows",
            rels.len(),
            affected_rows
        );
        Ok(())
    }

    pub(crate) fn upsert_triple(&self, triple: Triple) -> Result<RelationOrProperty, StoreError> {
        let subject_id = self.upsert_object(ObjectKey::from(&triple.subject))?;
        let predicate_id = self.upsert_predicate(triple.predicate.as_str())?;
        match triple.object {
            oxrdf::Term::NamedNode(object) => {
                let object_id = self.upsert_object(object.as_str())?;
                let relation = Relation::new(subject_id, predicate_id, object_id);
                self.create_relation_triple(relation)?;
                Ok(RelationOrProperty::Relation(relation))
            }
            oxrdf::Term::Literal(literal) => {
                let property = Property::new(
                    subject_id,
                    predicate_id,
                    literal.value().to_string(),
                    Some(literal.datatype().to_string()),
                );
                self.create_property_triple(property.clone());
                Ok(RelationOrProperty::Property(property))
            }
            oxrdf::Term::Triple(nested_triple) => {
                let nested = nested_triple.as_ref();
                let object_id = match self.upsert_triple(nested.clone())? {
                    RelationOrProperty::Property(property) => self.quote_property(property)?,
                    RelationOrProperty::Relation(relation) => self.quote_relation(relation)?,
                };
                let relation = Relation::new(subject_id, predicate_id, object_id);
                self.create_relation_triple(relation)?;
                Ok(RelationOrProperty::Relation(relation))
            }
            oxrdf::Term::BlankNode(_) => {
                return Err(StoreError::sparql_error(
                    "Blank Nodes are not supported yet",
                ));
            }
        }
    }

    // TODO: make this function actually use batching for the recursive insert
    pub(crate) fn batch_upsert_triples(&self, triples: &[Triple]) -> Result<(), StoreError> {
        let mut predicates: HashSet<NewPredicate> = HashSet::new();
        let mut objects: HashSet<ObjectKey> = HashSet::new();
        let mut direct_insert: HashSet<Triple> = HashSet::new();
        let mut quoted_triples: HashSet<Triple> = HashSet::new();
        for triple in triples {
            if let oxrdf::Term::Triple(_) = triple.object {
                quoted_triples.insert(triple.clone());
                continue;
            }
            objects.insert(ObjectKey::from(&triple.subject));
            predicates.insert(triple.predicate.as_str().into());
            match &triple.object {
                oxrdf::Term::NamedNode(named_node) => {
                    _ = objects.insert(ObjectKey::from(named_node))
                }
                oxrdf::Term::BlankNode(blank_node) => {
                    _ = objects.insert(ObjectKey::from(blank_node))
                }
                _ => (),
            }
            direct_insert.insert(triple.clone());
        }
        let predicate_ids = self.batch_upsert_predicates(predicates)?;
        let object_ids = self.batch_upsert_objects(objects)?;
        let mut properties: Vec<Property> = Vec::new();
        let mut relations: Vec<Relation> = Vec::new();
        for pending_triple in direct_insert.into_iter() {
            if let Some(&subject) = object_ids.get(&ObjectKey::from(&pending_triple.subject))
                && let Some(&predicate) = predicate_ids.get(pending_triple.predicate.as_str())
            {
                match &pending_triple.object {
                    oxrdf::Term::NamedNode(named_node) => {
                        if let Some(&object) = object_ids.get(&ObjectKey::from(named_node)) {
                            relations.push(Relation::new(subject, predicate, object));
                        } else {
                            log::error!("could not find id for the iri {}", named_node)
                        }
                    },
                    oxrdf::Term::BlankNode(bnode) => {
                        if let Some(&object) = object_ids.get(&ObjectKey::from(bnode)) {
                            relations.push(Relation::new(subject, predicate, object));
                        } else {
                            log::error!("could not find id for the iri {}", bnode)
                        }
                    }
                    oxrdf::Term::Literal(literal) => properties.push(Property::new(
                        subject,
                        predicate,
                        literal.value().to_string(),
                        Some(literal.datatype().as_str().to_string()),
                    )),
                    _ => panic!("rdf start recursive triples should not be able to get here"),
                }
            } else {
                log::error!("missing either subject or predicate id");
            }
        }
        self.batch_create_relation_triples(&relations)?;
        self.batch_create_property_triples(&properties)?;
        log::info!("Imported non recursive triples");
        for triple in quoted_triples.into_iter() {
            _ = self.upsert_triple(triple)?;
        }
        Ok(())
    }
}

pub(crate) fn query_relation_triples(
    conn: &mut PgPooledConnection,
    subject: TriplePosition,
    predicate: TriplePosition,
    object: TriplePosition,
) -> Result<Vec<Solution>, StoreError> {
    use crate::schema::objects;
    use crate::schema::predicates;
    use crate::schema::relations;

    let mut builder = SolutionBuilder::new();

    let (subject_objects, object_objects) =
        alias!(objects as subject_objects, objects as object_objects);

    let mut relation_query = relations::table
        .inner_join(subject_objects.on(relations::subject.eq(subject_objects.field(objects::id))))
        .inner_join(predicates::table.on(relations::predicate.eq(predicates::id)))
        .inner_join(object_objects.on(relations::object.eq(object_objects.field(objects::id))))
        .select((
            subject_objects.field(objects::value),
            predicates::iri,
            object_objects.fields(objects::value),
        ))
        .into_boxed();
    match subject {
        TriplePosition::Constant(val) => {
            relation_query = relation_query.filter(subject_objects.field(objects::value).eq(val));
        }
        TriplePosition::Variable(var) => {
            log::info!("VAR is: {var:?}");
            builder.subject(var)
        }
        TriplePosition::Bound(var, terms) => {
            let iris: Vec<String> = terms
                .iter()
                .filter_map(|t| match t {
                    model::triple::Term::Iri(s) => Some(s.clone()),
                    _ => None,
                })
                .collect();
            builder.subject(var);
            relation_query =
                relation_query.filter(subject_objects.field(objects::value).eq_any(iris))
        }
    }
    match predicate {
        TriplePosition::Constant(val) => {
            relation_query = relation_query.filter(predicates::iri.eq(val))
        }
        TriplePosition::Variable(val) => {
            log::info!("VAR is: {val:?}");
            builder.predicate(val)
        }
        TriplePosition::Bound(var, terms) => {
            let iris: Vec<String> = terms
                .iter()
                .filter_map(|t| match t {
                    model::triple::Term::Iri(s) => Some(s.clone()),
                    _ => None,
                })
                .collect();
            builder.predicate(var);
            relation_query = relation_query.filter(predicates::iri.eq_any(iris))
        }
    }
    match object {
        TriplePosition::Constant(val) => {
            relation_query = relation_query.filter(object_objects.field(objects::value).eq(val))
        }
        TriplePosition::Variable(var, ..) => {
            log::info!("VAR is: {var:?}");
            builder.object(var);
        }
        TriplePosition::Bound(var, terms) => {
            let iris: Vec<String> = terms
                .iter()
                .filter_map(|t| match t {
                    model::triple::Term::Iri(s) => Some(s.clone()),
                    _ => None,
                })
                .collect();
            builder.object(var);
            relation_query =
                relation_query.filter(object_objects.field(objects::value).eq_any(iris))
        }
    }

    Ok(builder.get_rel_solutions(relation_query.load::<(String, String, String)>(conn)?))
}

pub(crate) fn query_property_triples(
    conn: &mut PgPooledConnection,
    subject: TriplePosition,
    predicate: TriplePosition,
    object: TriplePosition,
    match_mode: LiteralMatchMode,
) -> Result<Vec<Solution>, StoreError> {
    use crate::schema::objects;
    use crate::schema::predicates;
    use crate::schema::properties;

    let mut builder = SolutionBuilder::new();

    let mut property_query = properties::table
        .inner_join(objects::table.on(properties::subject.eq(objects::id)))
        .inner_join(predicates::table.on(properties::predicate.eq(predicates::id)))
        .select((
            objects::value,
            predicates::iri,
            properties::literal_value,
            properties::literal_type,
        ))
        .into_boxed();
    match subject {
        TriplePosition::Constant(val) => {
            property_query = property_query.filter(objects::value.eq(val))
        }
        TriplePosition::Variable(var) => {
            log::info!("VAR is: {var:?}");
            builder.subject(var)
        }
        TriplePosition::Bound(var, terms) => {
            let iris: Vec<String> = terms
                .iter()
                .filter_map(|t| match t {
                    model::triple::Term::Iri(s) => Some(s.clone()),
                    _ => None,
                })
                .collect();
            builder.subject(var);
            property_query = property_query.filter(objects::value.eq_any(iris));
        }
    }
    match predicate {
        TriplePosition::Constant(val) => {
            property_query = property_query.filter(predicates::iri.eq(val))
        }
        TriplePosition::Variable(val) => {
            log::info!("VAR is: {val:?}");
            builder.predicate(val)
        }
        TriplePosition::Bound(var, terms) => {
            let iris: Vec<String> = terms
                .iter()
                .filter_map(|t| match t {
                    model::triple::Term::Iri(s) => Some(s.clone()),
                    _ => None,
                })
                .collect();
            builder.predicate(var);
            property_query = property_query.filter(predicates::iri.eq_any(iris))
        }
    }
    match object {
        TriplePosition::Constant(val) => {
            property_query = if match_mode == LiteralMatchMode::Exact {
                property_query.filter(properties::literal_value.eq(val))
            } else {
                let pattern = format!("%{val}%");
                property_query.filter(properties::literal_value.like(pattern))
            };
        }
        TriplePosition::Variable(var) => {
            log::info!("VAR is: {var:?}");
            builder.object(var);
        }
        TriplePosition::Bound(var, terms) => {
            let iris: Vec<String> = terms
                .iter()
                .filter_map(|t| match t {
                    model::triple::Term::Literal { value, .. } => Some(value.clone()),
                    _ => None,
                })
                .collect();
            builder.subject(var);
            property_query = property_query.filter(objects::value.eq_any(iris))
        }
    }
    Ok(builder.get_prop_solutions_typed(
        property_query.load::<(String, String, String, Option<String>)>(conn)?,
    ))
}
