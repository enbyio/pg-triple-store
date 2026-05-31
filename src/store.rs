use diesel::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};

use crate::StoreError;

pub type PgPool = Pool<ConnectionManager<PgConnection>>;
pub type PgPooledConnection = PooledConnection<ConnectionManager<PgConnection>>;

pub struct TripleStore {
    pool: PgPool,
}

impl TripleStore {
    pub fn new(database_url: &str) -> Result<Self, StoreError> {
        let manager = ConnectionManager::<PgConnection>::new(database_url);
        let pool = Pool::builder()
            .build(manager)
            .map_err(|e| StoreError::ConnectionError(e.to_string()))?;
        Ok(Self { pool })
    }

    pub(crate) fn conn(&self) -> Result<PgPooledConnection, StoreError> {
        self.pool
            .get()
            .map_err(|e| StoreError::ConnectionError(e.to_string()))
    }
}
