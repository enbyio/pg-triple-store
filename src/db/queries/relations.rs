use crate::db::models::query::{QueryOptions, Solution, SolutionBuilder, Term};
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
        options: QueryOptions,
    ) -> Result<Vec<Solution>, StoreError> {
        use crate::schema::objects;
        use crate::schema::predicates;
        use crate::schema::relations;

        let mut conn = self.conn()?;

        let mut builder = SolutionBuilder::new();

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
        match query.subject {
            crate::db::models::triple::TriplePosition::Constant(val) => {
                relation_query = relation_query.filter(subject_objects.field(objects::iri).eq(val));
            }
            crate::db::models::triple::TriplePosition::Variable(var) => builder.subject(var),
            crate::db::models::triple::TriplePosition::Bound(var, terms) => {
                let iris: Vec<String> = terms
                    .iter()
                    .filter_map(|t| match t {
                        Term::Iri(s) => Some(s.clone()),
                        _ => None,
                    })
                    .collect();
                builder.subject(var);
                relation_query =
                    relation_query.filter(subject_objects.field(objects::iri).eq_any(iris))
            }
        }
        match query.predicate {
            crate::db::models::triple::TriplePosition::Constant(val) => {
                relation_query = relation_query.filter(predicates::iri.eq(val))
            }
            crate::db::models::triple::TriplePosition::Variable(val) => builder.predicate(val),
            crate::db::models::triple::TriplePosition::Bound(var, terms) => {
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
        match query.object {
            crate::db::models::triple::TriplePosition::Constant(val) => {
                relation_query = relation_query.filter(object_objects.field(objects::iri).eq(val))
            }
            crate::db::models::triple::TriplePosition::Variable(var) => {
                builder.object(var);
            }
            crate::db::models::triple::TriplePosition::Bound(var, terms) => {
                let iris: Vec<String> = terms
                    .iter()
                    .filter_map(|t| match t {
                        Term::Iri(s) => Some(s.clone()),
                        _ => None,
                    })
                    .collect();
                builder.object(var);
                relation_query =
                    relation_query.filter(object_objects.field(objects::iri).eq_any(iris))
            }
        }
        if let Some(lim) = options.limit {
            relation_query = relation_query.limit(lim as i64);
        }
        if options.offset > 0 {
            relation_query = relation_query.offset(options.offset as i64);
        }

        let result =
            builder.get_rel_solutions(relation_query.load::<(String, String, String)>(&mut conn)?);
        Ok(result)

        //Ok(relation_query.load::<(String, String, String)>(&mut conn)?)
    }
}
