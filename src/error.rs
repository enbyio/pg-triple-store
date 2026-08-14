use std::fmt::Display;
use std::io;
use std::panic::Location;

#[derive(Debug)]
pub struct StoreError {
    pub kind: StoreErrorKind,
    pub msg: String,
    pub location: &'static Location<'static>,
}

#[derive(Debug)]
pub enum StoreErrorKind {
    DatabaseError(DatabaseError),
    DataError,
    SparqlError,
    IOError,
}

#[derive(Debug)]
pub enum DatabaseError {
    ConfigurationError,
    ConnectionError,
    MigrationError,
    DieselError,
}

impl StoreError {
    #[track_caller]
    pub fn db_error(kind: DatabaseError, msg: impl Display) -> Self {
        log::error!("DB_ERROR({:?}): {msg}", kind);
        Self {
            kind: StoreErrorKind::DatabaseError(kind),
            msg: msg.to_string(),
            location: Location::caller(),
        }
    }

    #[track_caller]
    pub fn sparql_error(msg: impl Display) -> Self {
        log::error!("SPARQL_ERROR: {msg}");
        Self {
            kind: StoreErrorKind::SparqlError,
            msg: msg.to_string(),
            location: Location::caller(),
        }
    }

    #[track_caller]
    pub fn data_error(msg: impl Display) -> Self {
        log::error!("DATA_ERROR: {msg}");
        Self {
            kind: StoreErrorKind::DataError,
            msg: msg.to_string(),
            location: Location::caller(),
        }
    }

    #[track_caller]
    pub fn io_error(msg: impl Display) -> Self {
        log::error!("IO_ERROR: {msg}");
        Self {
            kind: StoreErrorKind::IOError,
            msg: msg.to_string(),
            location: Location::caller(),
        }
    }
}

impl From<oxrdf::IriParseError> for StoreError {
    fn from(value: oxrdf::IriParseError) -> Self {
        StoreError::data_error(format!("IriParseError: {value}"))
    }
}

impl From<diesel::result::Error> for StoreError {
    fn from(value: diesel::result::Error) -> Self {
        StoreError::db_error(DatabaseError::DieselError, value)
    }
}

impl From<spargebra::SparqlSyntaxError> for StoreError {
    fn from(value: spargebra::SparqlSyntaxError) -> Self {
        StoreError::sparql_error(format!("Error whilst parsing Sparql Input {value}"))
    }
}

impl From<io::Error> for StoreError {
    fn from(value: io::Error) -> Self {
        StoreError::io_error(format!("IOError {value}"))
    }
}
