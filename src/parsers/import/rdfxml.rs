use oxrdf::Triple;
use oxrdfxml::RdfXmlParser;

use crate::error::StoreError;
use crate::model::prefix::Prefix;
use crate::store::TripleStore;

impl TripleStore {
    /// Import rdf-xml data into the triple store
    pub fn import_rdfxml_data(&self, data: String) -> Result<(), StoreError> {
        let mut parser = RdfXmlParser::new().for_reader(data.as_bytes());
        let mut triples: Vec<Triple> = Vec::new();
        for triple in parser.by_ref() {
            match triple {
                Ok(triple) => triples.push(triple),
                Err(e) => log::error!("Error: {}", e),
            }
        }
        let prefixes: Vec<Prefix> = parser
            .prefixes()
            .map(|(s1, s2)| Prefix::new(s2.to_string(), s1.to_string()))
            .collect();
        self.import_prefixes(prefixes)?;
        log::info!("Found {} triples", triples.len());
        self.batch_upsert_triples(&triples)
    }

    pub fn import_rdfxml_file(&self, path: &str) -> Result<(), StoreError> {
        let data = std::fs::read_to_string(path)?;
        self.import_rdfxml_data(data)
    }
}
