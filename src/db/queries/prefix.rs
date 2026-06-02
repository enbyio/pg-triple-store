use diesel::upsert::excluded;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};

use crate::db::models::prefix::Prefix;
use crate::store::TripleStore;
use crate::StoreError;

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

    pub fn get_absolute_form(&self, iri: String) -> Result<String, StoreError> {
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

    pub fn get_short_form(&self, iri: String) -> Result<String, StoreError> {
        let mut trimmed = iri.trim();
        if trimmed.starts_with('<') && trimmed.ends_with('>') {
            trimmed = &trimmed[1..trimmed.len() - 1];
        }
        let mut conn = self.conn()?;
        let split_pos = trimmed.rfind(['#', '/']).ok_or(StoreError::DataError(
            "failed to find # or / absolute iri seems to be invalid".to_string(),
        ))?;
        let (long_prefix, value) = (&trimmed[..=split_pos], &trimmed[split_pos + 1..]);
        let short_namespace = prefixes
            .filter(namespace.eq(long_prefix))
            .select(prefix)
            .first::<String>(&mut conn)?;
        Ok(format!("{}:{}", short_namespace, value))
    }

    pub(crate) fn normalize_iri(&self, raw_iri: &str) -> Result<String, StoreError> {
        let trimmed = raw_iri.trim();
        if trimmed.starts_with('<') && trimmed.ends_with('>') {
            return Ok(trimmed[1..trimmed.len() - 1].to_string());
        }
        if trimmed.starts_with("http://")
            || trimmed.starts_with("https://")
            || trimmed.starts_with("urn:")
            || trimmed.starts_with("ftp://")
        {
            return Ok(trimmed.to_string());
        }
        if let Some((prefix_part, _)) = trimmed.split_once(':')
            && !prefix_part.contains('/') {
                return self.get_absolute_form(trimmed.to_string());
            }
        Err(StoreError::DataError(format!(
            "Cannot normalize IRI: '{}'",
            raw_iri
        )))
    }
}
