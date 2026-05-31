use log::{error, info};
use oxrdf::Triple;
use oxttl::TurtleParser;

use crate::db::models::prefix::Prefix;
use crate::store::TripleStore;
use crate::StoreError;

impl TripleStore {
    pub fn import_turtle_data(&mut self, data: String) -> Result<(), StoreError> {
        let mut parser = TurtleParser::new().for_reader(data.as_bytes());
        let mut triples: Vec<Triple> = Vec::new();
        for triple in parser.by_ref() {
            match triple {
                Ok(triple) => triples.push(triple),
                Err(e) => error!("Error: {}", e),
            }
        }
        let prefixes: Vec<Prefix> = parser
            .prefixes()
            .map(|(s1, s2)| Prefix::new(s2.to_string(), s1.to_string()))
            .collect();
        self.import_prefixes(prefixes)?;
        info!("Found {} triples", triples.len());
        self.batch_upsert_triples(&triples)
    }
}
