use diesel::upsert::excluded;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};

use crate::StoreError;
use crate::db::models::prefix::Prefix;
use crate::store::TripleStore;

use crate::schema::prefixes::dsl::*;

impl TripleStore {
    pub fn import_prefixes(&mut self, prefix_list: Vec<Prefix>) -> Result<(), StoreError> {
        let mut conn = self.conn()?;
        diesel::insert_into(prefixes)
            .values(prefix_list)
            .on_conflict(namespace)
            .do_update()
            .set(prefix.eq(excluded(prefix)))
            .execute(&mut conn)?;
        Ok(())
    }

    pub fn get_absolute_form(&mut self, iri: String) -> Result<String, StoreError> {
        let mut conn = self.conn()?;
        let short_prefix = iri.split_once(":").ok_or(StoreError::DataError(
            "short iri does not contain a : and is invalid".to_string(),
        ))?;
        println!("{} - {}", short_prefix.0, short_prefix.1);
        let long_prefix = prefixes
            .filter(prefix.eq(short_prefix.0))
            .select(namespace)
            .first::<String>(&mut conn)?;
        Ok(format!("{}{}", long_prefix, short_prefix.1))
    }

    pub fn get_short_form(&mut self, iri: String) -> Result<String, StoreError> {
        let mut conn = self.conn()?;
        let split_pos = iri.rfind(['#', '/']).ok_or(StoreError::DataError(
            "failed to find # or / absolute iri seems to be invalid".to_string(),
        ))?;
        let (long_prefix, value) = (&iri[..=split_pos], &iri[split_pos + 1..]);
        let short_namespace = prefixes
            .filter(namespace.eq(long_prefix))
            .select(prefix)
            .first::<String>(&mut conn)?;
        Ok(format!("{}:{}", short_namespace, value))
    }
}
