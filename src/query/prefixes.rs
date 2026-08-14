use diesel::upsert::excluded;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};

use crate::error::StoreError;
use crate::model::prefix::Prefix;
use crate::schema::prefixes::dsl::*;
use crate::store::TripleStore;

impl TripleStore {
    pub(crate) fn import_prefixes(&mut self, prefix_list: Vec<Prefix>) -> Result<(), StoreError> {
        let mut conn = self.conn()?;
        diesel::insert_into(prefixes)
            .values(prefix_list)
            .on_conflict(namespace)
            .do_update()
            .set(prefix.eq(excluded(prefix)))
            .execute(&mut conn)?;
        Ok(())
    }

    pub(crate) fn upsert_prefix(&mut self, pfx: String, nsp: String) -> Result<(), StoreError> {
        let prefix_new = Prefix::new(nsp, pfx);
        let mut conn = self.conn()?;
        diesel::insert_into(prefixes)
            .values(prefix_new)
            .on_conflict(namespace)
            .do_update()
            .set(prefix.eq(excluded(prefix)))
            .execute(&mut conn)?;
        Ok(())
    }

    pub(crate) fn get_absolute_form(&self, iri: String) -> Result<String, StoreError> {
        let mut conn = self.conn()?;
        let short_prefix = iri.split_once(":").ok_or(StoreError::data_error(
            "short iri does not contain a : and is invalid",
        ))?;
        println!("{} - {}", short_prefix.0, short_prefix.1);
        let long_prefix = prefixes
            .filter(prefix.eq(short_prefix.0))
            .select(namespace)
            .first::<String>(&mut conn)?;
        Ok(format!("{}{}", long_prefix, short_prefix.1))
    }

    pub(crate) fn get_all_prefixes(&mut self) -> Result<Vec<(String, String)>, StoreError> {
        let mut conn = self.conn()?;
        Ok(prefixes
            .select((namespace, prefix))
            .load::<(String, String)>(&mut conn)?)
    }

    pub(crate) fn shorten_iri(&mut self, iri: String) -> Result<String, StoreError> {
        let mut best: Option<(String, String)> = None;
        let mut longest_fit: usize = 0;
        for (ns, pfx) in self.get_all_prefixes()? {
            if iri.starts_with(ns.as_str()) && ns.len() > longest_fit {
                best = Some((ns.clone(), pfx.clone()));
                longest_fit = ns.len();
            }
        }
        Ok(match best {
            Some((ns, pfx)) => {
                let local = &iri[ns.len()..];
                if local.is_empty() {
                    iri
                } else {
                    format!("{}:{}", pfx, local)
                }
            }
            None => iri.to_string(),
        })
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
            && !prefix_part.contains('/')
        {
            return self.get_absolute_form(trimmed.to_string());
        }
        Err(StoreError::data_error(format!(
            "Cannot normalize IRI: '{raw_iri}'"
        )))
    }
}
