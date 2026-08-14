use diesel::{ExpressionMethods, JoinOnDsl, QueryDsl, RunQueryDsl};
use oxrdf::{Literal, NamedNode, NamedOrBlankNode, Term, Triple};

use crate::error::StoreError;
use crate::schema::properties::{literal_type, literal_value};
use crate::store::TripleStore;

impl TripleStore {
    /// returns all triples
    pub(crate) fn describe_object_id(&mut self, object_id: i64) -> Result<Vec<Triple>, StoreError> {
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
}
