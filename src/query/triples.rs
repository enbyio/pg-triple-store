use std::collections::HashSet;

use diesel::{ExpressionMethods, JoinOnDsl, QueryDsl, RunQueryDsl, TextExpressionMethods, alias};
use oxrdf::Triple;

use crate::error::StoreError;
use crate::model::object::NewObject;
use crate::model::predicate::NewPredicate;
use crate::model::property::Property;
use crate::model::relation::Relation;
use crate::model::triple::{AsIri, LiteralMatchMode, Term, TriplePosition};
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

    pub(crate) fn upsert_triple(&self, triple: Triple) -> Result<(), StoreError> {
        let subject_id = self.upsert_object(triple.subject.as_iri()?)?;
        let predicate_id = self.upsert_predicate(triple.predicate.as_str())?;
        match triple.object {
            oxrdf::Term::NamedNode(object) => {
                let object_id = self.upsert_object(object.as_str())?;
                self.create_relation_triple(Relation::new(subject_id, predicate_id, object_id))?;
            }
            oxrdf::Term::Literal(literal) => {
                self.create_property_triple(Property::new(
                    subject_id,
                    predicate_id,
                    literal.value().to_string(),
                    Some(literal.datatype().to_string()),
                ))?;
            }
            _ => {
                return Err(StoreError::sparql_error(
                    "Blank Node and Triples in Triples are not supported yet",
                ));
            }
        }
        Ok(())
    }

    pub(crate) fn batch_upsert_triples(&self, triples: &[Triple]) -> Result<(), StoreError> {
        let mut predicates: HashSet<NewPredicate> = HashSet::new();
        let mut objects: HashSet<NewObject> = HashSet::new();
        for triple in triples {
            objects.insert(triple.subject.as_iri()?.into());
            predicates.insert(triple.predicate.as_str().into());
            if let oxrdf::Term::NamedNode(val) = &triple.object {
                objects.insert(val.as_str()./*to_string().as_str().*/into());
            }
        }
        log::info!("Found {} unique objects", objects.len());
        log::info!("Found {} unique predicates", predicates.len());
        let predicate_ids = self.batch_upsert_predicates(predicates)?;
        let object_ids = self.batch_upsert_objects(objects)?;
        let mut relations: Vec<Relation> = Vec::new();
        let mut properties: Vec<Property> = Vec::new();
        for triple in triples {
            if let Some(&subject) = object_ids.get(triple.subject.as_iri()?)
                && let Some(&predicate) = predicate_ids.get(triple.predicate.as_str())
            {
                match &triple.object {
                    oxrdf::Term::NamedNode(named_node) => {
                        if let Some(&object) = object_ids.get(named_node.as_str()) {
                            relations.push(Relation::new(subject, predicate, object));
                        } else {
                            log::error!("could not find id for the iri {}", named_node)
                        }
                    }
                    oxrdf::Term::Literal(literal) => properties.push(Property::new(
                        subject,
                        predicate,
                        literal.value().to_string(),
                        Some(literal.datatype().as_str().to_string()),
                    )),
                    _ => log::error!("Blank Node and Triples in Triples are not supported yet"),
                }
            } else {
                log::error!("missing either subject or predicate id");
            }
        }
        self.batch_create_relation_triples(&relations)?;
        self.batch_create_property_triples(&properties)?;
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
        TriplePosition::Variable(var) => builder.subject(var),
        TriplePosition::Bound(var, terms) => {
            let iris: Vec<String> = terms
                .iter()
                .filter_map(|t| match t {
                    Term::Iri(s) => Some(s.clone()),
                    _ => None,
                })
                .collect();
            builder.subject(var);
            relation_query = relation_query.filter(subject_objects.field(objects::value).eq_any(iris))
        }
    }
    match predicate {
        TriplePosition::Constant(val) => {
            relation_query = relation_query.filter(predicates::iri.eq(val))
        }
        TriplePosition::Variable(val) => builder.predicate(val),
        TriplePosition::Bound(var, terms) => {
            let iris: Vec<String> = terms
                .iter()
                .filter_map(|t| match t {
                    Term::Iri(s) => Some(s.clone()),
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
        TriplePosition::Variable(var) => {
            builder.object(var);
        }
        TriplePosition::Bound(var, terms) => {
            let iris: Vec<String> = terms
                .iter()
                .filter_map(|t| match t {
                    Term::Iri(s) => Some(s.clone()),
                    _ => None,
                })
                .collect();
            builder.object(var);
            relation_query = relation_query.filter(object_objects.field(objects::value).eq_any(iris))
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
        TriplePosition::Variable(var) => builder.subject(var),
        TriplePosition::Bound(var, terms) => {
            let iris: Vec<String> = terms
                .iter()
                .filter_map(|t| match t {
                    Term::Iri(s) => Some(s.clone()),
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
        TriplePosition::Variable(val) => builder.predicate(val),
        TriplePosition::Bound(var, terms) => {
            let iris: Vec<String> = terms
                .iter()
                .filter_map(|t| match t {
                    Term::Iri(s) => Some(s.clone()),
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
            builder.object(var);
        }
        TriplePosition::Bound(var, terms) => {
            let iris: Vec<String> = terms
                .iter()
                .filter_map(|t| match t {
                    Term::Literal { value, .. } => Some(value.clone()),
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
