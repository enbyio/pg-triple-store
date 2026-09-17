use std::collections::HashMap;

use diesel::sql_types::BigInt;
use diesel::{
    sql_query, BoolExpressionMethods, Connection, ExpressionMethods, OptionalExtension, QueryDsl,
    RunQueryDsl,
};

use crate::error::StoreError;
use crate::model::property::Property;
use crate::model::relation::Relation;
use crate::model::triple::RelationOrProperty;
use crate::store::TripleStore;

// functions are internal so for performance reason it is assumed that the property / relation is already in the appropriate table
impl TripleStore {
    // TODO: implement actual batching
    pub(crate) fn batch_quote_triple(
        &self,
        triples: Vec<RelationOrProperty>,
    ) -> Result<HashMap<RelationOrProperty, i64>, StoreError> {
        let mut map = HashMap::<RelationOrProperty, i64>::new();
        for triple in triples {
            let id = self.quote_triple(triple.clone())?;
            map.entry(triple).or_insert(id);
        }
        Ok(map)
    }

    pub(crate) fn quote_triple(&self, triple: RelationOrProperty) -> Result<i64, StoreError> {
        match triple {
            RelationOrProperty::Property(property) => self.quote_property(property),
            RelationOrProperty::Relation(relation) => self.quote_relation(relation),
        }
    }

    pub(crate) fn quote_property(&self, property: Property) -> Result<i64, StoreError> {
        use crate::schema::quoted_properties::dsl::*;
        let mut conn = self.conn()?;
        conn.transaction(|conn| {
            sql_query("SELECT pg_advisory_xact_lock($1)")
                .bind::<BigInt, _>(property.hash())
                .execute(conn)?;
            if let Some(existing) = quoted_properties
                .filter(
                    subject
                        .eq(&property.subject)
                        .and(predicate.eq(&property.predicate))
                        .and(literal_value.eq(&property.literal_value)),
                )
                .select(id)
                .first::<i64>(conn)
                .optional()?
            {
                return Ok(existing);
            }

            let new_id: i64 = self
                .add_entity_with_session(crate::model::entity::EntityType::QuotedProperty, conn)?;

            diesel::insert_into(quoted_properties)
                .values(property.quote(new_id))
                .execute(conn)?;

            Ok(new_id)
        })
    }

    pub(crate) fn quote_relation(&self, relation: Relation) -> Result<i64, StoreError> {
        use crate::schema::quoted_relations::dsl::*;
        let mut conn = self.conn()?;
        conn.transaction(|conn| {
            sql_query("SELECT pg_advisory_xact_lock($1)")
                .bind::<BigInt, _>(relation.hash())
                .execute(conn)?;
            if let Some(existing) = quoted_relations
                .filter(
                    subject
                        .eq(&relation.subject)
                        .and(predicate.eq(&relation.predicate))
                        .and(object.eq(&relation.object)),
                )
                .select(id)
                .first::<i64>(conn)
                .optional()?
            {
                return Ok(existing);
            }

            let new_id: i64 = self
                .add_entity_with_session(crate::model::entity::EntityType::QuotedRelation, conn)?;

            diesel::insert_into(quoted_relations)
                .values(relation.quote(new_id))
                .execute(conn)?;

            Ok(new_id)
        })
    }
}
