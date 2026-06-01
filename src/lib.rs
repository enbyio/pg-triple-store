pub mod db;
pub mod parsers;
pub mod schema;
pub mod store;

#[derive(Debug)]
pub enum StoreError {
    GeneralError(String),
    DatabaseError(String),
    DataError(String),
    ConnectionError(String),
    MigrationError(String),
    SparqlSyntaxError(String),
    UnsupportedInputData,
}

impl From<diesel::result::Error> for StoreError {
    fn from(value: diesel::result::Error) -> Self {
        StoreError::DatabaseError(value.to_string())
    }
}

impl From<diesel::r2d2::Error> for StoreError {
    fn from(value: diesel::r2d2::Error) -> Self {
        StoreError::ConnectionError(value.to_string())
    }
}

impl From<diesel::r2d2::PoolError> for StoreError {
    fn from(value: diesel::r2d2::PoolError) -> Self {
        StoreError::ConnectionError(value.to_string())
    }
}

impl From<spargebra::SparqlSyntaxError> for StoreError {
    fn from(value: spargebra::SparqlSyntaxError) -> Self {
        StoreError::SparqlSyntaxError(value.to_string())
    }
}

impl From<reqwest::Error> for StoreError {
    fn from(value: reqwest::Error) -> Self {
        StoreError::ConnectionError(format!("Reqwest Error: {}", value))
    }
}
