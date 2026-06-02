use diesel::{ExpressionMethods, JoinOnDsl, QueryDsl, RunQueryDsl, TextExpressionMethods};
use log::{debug, info};
use oxrdf::Literal;

use crate::db::models::property::{LiteralMatchMode, Property, PropertyTripleQuery};
use crate::db::models::query::QueryOptions;
use crate::db::models::triple::TripleQueryResult;
use crate::store::TripleStore;
use crate::StoreError;

impl TripleStore {
    pub fn create_property_triple(
        &mut self,
        subject_id: i64,
        predicate_id: i64,
        object: Literal,
    ) -> Result<(), StoreError> {
        use crate::schema::properties::dsl::*;
        let mut conn = self.conn()?;
        let object_value = object.to_string();
        let object_type = object.datatype().to_string();
        let affected_rows = diesel::insert_into(properties)
            .values((
                subject.eq(subject_id),
                predicate.eq(predicate_id),
                literal_value.eq(&object_value),
                literal_type.eq(&object_type),
            ))
            .on_conflict((subject, predicate, literal_value))
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

    pub fn batch_create_property_triple(&mut self, triples: &[Property]) -> Result<(), StoreError> {
        use crate::schema::properties::dsl::*;
        let mut conn = self.conn()?;
        let affected_rows = diesel::insert_into(properties)
            .values(triples)
            .on_conflict_do_nothing()
            .execute(&mut conn)?;
        info!(
            "BATCH INSERT PROPERTIES for {} elements affected {} rows",
            triples.len(),
            affected_rows
        );
        Ok(())
    }

    pub fn query_property_triples_joined(
        &mut self,
        query: PropertyTripleQuery,
        options: QueryOptions,
    ) -> Result<Vec<TripleQueryResult>, StoreError> {
        use crate::schema::objects;
        use crate::schema::predicates;
        use crate::schema::properties;

        let mut conn = self.conn()?;

        let mut property_query = properties::table
            .inner_join(objects::table.on(properties::subject.eq(objects::id)))
            .inner_join(predicates::table.on(properties::predicate.eq(predicates::id)))
            .select((objects::iri, predicates::iri, properties::literal_value))
            .into_boxed();
        if let Some(subject_iri) = query.subject {
            property_query = property_query.filter(objects::iri.eq(format!("<{subject_iri}>")));
        }

        if let Some(predicate_iri) = query.predicate {
            property_query =
                property_query.filter(predicates::iri.eq(format!("<{predicate_iri}>")));
        }

        if let Some(value) = query.literal_value {
            property_query = if query.literal_match_mode == LiteralMatchMode::Exact {
                property_query.filter(properties::literal_value.eq(value))
            } else {
                let pattern = format!("%{value}%");
                property_query.filter(properties::literal_value.like(pattern))
            };
        }

        if let Some(lim) = options.limit {
            property_query = property_query.limit(lim as i64);
        }
        if options.offset > 0 {
            property_query = property_query.offset(options.offset as i64);
        }

        let result = property_query.load::<(String, String, String)>(&mut conn)?;
        Ok(result
            .into_iter()
            .map(TripleQueryResult::property)
            .collect())
    }
}
