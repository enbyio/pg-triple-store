use oxrdf::Triple;
use oxrdfxml::RdfXmlSerializer;

use crate::error::StoreError;
use crate::store::TripleStore;

impl TripleStore {
    /// generate an rdfxml string from a list of Triples
    /// intended use is for the List of Triples extracted from a QueryResult::Graph(...)
    /// this may be finalised with some changes to how the api works in the near future
    pub fn export_as_rdfxml(&self, triples: &[Triple]) -> Result<String, StoreError> {
        let mut serializer = RdfXmlSerializer::new().for_writer(Vec::new());

        for triple in triples {
            serializer.serialize_triple(triple)?;
        }
        let bytes: Vec<u8> = serializer.finish()?;
        Ok(String::from_utf8_lossy(&bytes).to_string())
    }
}
