use crate::db::models::relation::{Relation, RelationTripleQuery};
use crate::store::TripleStore;
use crate::StoreError;
use diesel::{alias, ExpressionMethods, JoinOnDsl, QueryDsl, RunQueryDsl};
use log::{debug, info};

impl TripleStore {
    pub fn create_relation_triple(
        &mut self,
        subject_id: i64,
        predicate_id: i64,
        object_id: i64,
    ) -> Result<(), StoreError> {
        use crate::schema::relations::dsl::*;
        let mut conn = self.conn()?;
        let affected_rows = diesel::insert_into(relations)
            .values((
                subject.eq(subject_id),
                predicate.eq(predicate_id),
                object.eq(object_id),
            ))
            .on_conflict((subject, predicate, object))
            .do_nothing()
            .execute(&mut conn)?;

        if affected_rows == 0 {
            debug!(
                "Duplicate property triple skipped for subject={}, predicate={}",
                subject_id, predicate_id
            );
        } else {
            debug!(
                "Rows affected from property triple insert: {}",
                affected_rows
            );
        }
        Ok(())
    }

    pub fn batch_create_relation_triple(&mut self, triples: &[Relation]) -> Result<(), StoreError> {
        use crate::schema::relations::dsl::*;
        let mut conn = self.conn()?;
        let affected_rows = diesel::insert_into(relations)
            .values(triples)
            .on_conflict_do_nothing()
            .execute(&mut conn)?;
        info!(
            "BATCH INSERT RELATIONS for {} elements affected {} rows",
            triples.len(),
            affected_rows
        );
        Ok(())
    }

    pub fn query_relation_triples_joined(
        &mut self,
        query: RelationTripleQuery,
    ) -> Result<Vec<(String, String, String)>, StoreError> {
        use crate::schema::objects;
        use crate::schema::predicates;
        use crate::schema::relations;

        let mut conn = self.conn()?;

        let (subject_objects, object_objects) =
            alias!(objects as subject_objects, objects as object_objects);

        let mut relation_query = relations::table
            .inner_join(
                subject_objects.on(relations::subject.eq(subject_objects.field(objects::id))),
            )
            .inner_join(predicates::table.on(relations::predicate.eq(predicates::id)))
            .inner_join(object_objects.on(relations::object.eq(object_objects.field(objects::id))))
            .select((
                subject_objects.field(objects::iri),
                predicates::iri,
                object_objects.fields(objects::iri),
            ))
            .into_boxed();

        if let Some(subject_iri) = query.subject {
            println!("subject is {subject_iri}");
            relation_query = relation_query.filter(
                subject_objects
                    .field(objects::iri)
                    .eq(format!("<{subject_iri}>")),
            );
        }

        if let Some(predicate_iri) = query.predicate {
            println!("predicate is {predicate_iri}");
            relation_query = relation_query.filter(predicates::iri.eq(format!("<{predicate_iri}>")))
        }

        if let Some(object_iri) = query.object {
            println!("object is {object_iri}");
            relation_query = relation_query.filter(
                object_objects
                    .field(objects::iri)
                    .eq(format!("<{object_iri}>")),
            )
        }

        Ok(relation_query.load::<(String, String, String)>(&mut conn)?)
    }
}
