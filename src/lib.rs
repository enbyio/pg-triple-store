pub mod db;
pub mod parsers;
pub mod schema;
pub mod store;

use diesel::result::Error;

#[derive(Debug)]
pub enum StoreError {
    GeneralError(String),
    DatabaseError(String),
    DataError(String),
    ConnectionError(String),
    UnsupportedInputData,
}

impl From<Error> for StoreError {
    fn from(value: Error) -> Self {
        StoreError::DatabaseError(value.to_string())
    }
}
