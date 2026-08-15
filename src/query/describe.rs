use diesel::{alias, ExpressionMethods, JoinOnDsl, QueryDsl, RunQueryDsl};
use oxrdf::{Literal, NamedNode, NamedOrBlankNode, Term, Triple};

use crate::error::StoreError;
use crate::schema::properties::{literal_type, literal_value};
use crate::schema::relations;
use crate::store::TripleStore;

impl TripleStore {
    /// returns all triples
    pub(crate) fn describe_object_id(&self, object_id: i64) -> Result<Vec<Triple>, StoreError> {
        use crate::schema::objects;
        use crate::schema::predicates;
        use crate::schema::properties;
        use crate::schema::relations;
        let mut conn = self.conn()?;
        let mut results: Vec<Triple> = Vec::new();
        // Relations with the object in the subject
        let object_iri: String = objects::table
            .select(objects::iri)
            .filter(objects::id.eq(object_id))
            .first(&mut conn)?;
        let rels_subject = relations::table
            .inner_join(predicates::table.on(relations::predicate.eq(predicates::id)))
            .inner_join(objects::table.on(relations::object.eq(objects::id)))
            .select((predicates::iri, objects::iri))
            .filter(relations::subject.eq(object_id))
            .load::<(String, String)>(&mut conn)?;
        let subject = NamedOrBlankNode::NamedNode(NamedNode::new(object_iri.clone())?);
        for rel in rels_subject {
            results.push(Triple {
                subject: subject.clone(),
                predicate: NamedNode::new(rel.0)?,
                object: Term::NamedNode(NamedNode::new(rel.1)?),
            });
        }
        // Relations with the object in the object field
        let rels_object = relations::table
            .inner_join(objects::table.on(relations::subject.eq(objects::id)))
            .inner_join(predicates::table.on(relations::predicate.eq(predicates::id)))
            .select((objects::iri, predicates::iri))
            .filter(relations::object.eq(object_id))
            .load::<(String, String)>(&mut conn)?;
        let object = Term::NamedNode(NamedNode::new(object_iri)?);
        for rel in rels_object {
            results.push(Triple {
                subject: NamedOrBlankNode::NamedNode(NamedNode::new(rel.0)?),
                predicate: NamedNode::new(rel.1)?,
                object: object.clone(),
            });
        }

        // Properties
        let props = properties::table
            .inner_join(predicates::table.on(properties::predicate.eq(predicates::id)))
            .select((predicates::iri, literal_value, literal_type))
            .filter(properties::subject.eq(object_id))
            .load::<(String, String, Option<String>)>(&mut conn)?;
        for property in props {
            let literal = match property.2 {
                Some(value) => Literal::new_typed_literal(property.1, NamedNode::new(value)?),
                None => Literal::new_simple_literal(property.1),
            };
            results.push(Triple {
                subject: subject.clone(),
                predicate: NamedNode::new(property.0)?,
                object: Term::Literal(literal),
            });
        }
        Ok(results)
    }

    pub(crate) fn describe_predicate_id(
        &self,
        predicate_id: i64,
    ) -> Result<Vec<Triple>, StoreError> {
        use crate::schema::objects;
        use crate::schema::predicates;
        use crate::schema::properties;
        let (subject_objects, object_objects) =
            alias!(objects as subject_objects, objects as object_objects);
        let mut conn = self.conn()?;
        let mut results = Vec::new();
        let predicate_iri: String = predicates::table
            .select(predicates::iri)
            .filter(predicates::id.eq(predicate_id))
            .first(&mut conn)?;
        let pred_node = NamedNode::new(predicate_iri)?;

        let rels = relations::table
            .inner_join(
                subject_objects.on(relations::subject.eq(subject_objects.field(objects::id))),
            )
            .inner_join(object_objects.on(relations::object.eq(object_objects.fields(objects::id))))
            .select((
                subject_objects.field(objects::iri),
                object_objects.fields(objects::iri),
            ))
            .filter(relations::predicate.eq(predicate_id))
            .load::<(String, String)>(&mut conn)?;
        for relation in rels {
            results.push(Triple {
                subject: NamedOrBlankNode::NamedNode(NamedNode::new(relation.0)?),
                predicate: pred_node.clone(),
                object: Term::NamedNode(NamedNode::new(relation.1)?),
            });
        }

        let props = properties::table
            .inner_join(objects::table.on(properties::subject.eq(objects::id)))
            .select((
                objects::iri,
                properties::literal_value,
                properties::literal_type,
            ))
            .filter(properties::predicate.eq(predicate_id))
            .load::<(String, String, Option<String>)>(&mut conn)?;

        for property in props {
            let literal = match property.2 {
                Some(value) => Literal::new_typed_literal(property.1, NamedNode::new(value)?),
                None => Literal::new_simple_literal(property.1),
            };
            results.push(Triple {
                subject: NamedOrBlankNode::NamedNode(NamedNode::new(property.0)?),
                predicate: pred_node.clone(),
                object: Term::Literal(literal),
            });
        }
        Ok(results)
    }

    pub(crate) fn get_list_of_relation_subjects(
        &self,
        predicate: i64,
        object: i64,
    ) -> Result<Vec<(String, i64)>, StoreError> {
        use crate::schema::objects;
        use crate::schema::relations;
        let mut conn = self.conn()?;
        Ok(relations::table
            .inner_join(objects::table.on(relations::subject.eq(objects::id)))
            .select((objects::iri, relations::object))
            .filter(relations::predicate.eq(predicate))
            .filter(relations::object.eq(object))
            .load::<(String, i64)>(&mut conn)?)
    }
}
