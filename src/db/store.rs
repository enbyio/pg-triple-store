use std::time::Duration;

use diesel::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};

use crate::db::error::{DatabaseError, StoreError};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

pub(crate) type PgPool = Pool<ConnectionManager<PgConnection>>;
pub(crate) type PgPooledConnection = PooledConnection<ConnectionManager<PgConnection>>;

pub struct TripleStore {
    pool: PgPool,
}

impl TripleStore {
    pub fn new(database_url: &str) -> Result<Self, StoreError> {
        let manager = ConnectionManager::<PgConnection>::new(database_url);
        let pool = Pool::builder()
            .connection_timeout(Duration::from_secs(10))
            .max_size(10)
            .min_idle(Some(1))
            .build(manager)
            .map_err(|e| StoreError::db_error(DatabaseError::ConnectionError, e))?;
        let mut db = Self { pool };
        db.migrate()?;
        Ok(db)
    }

    pub fn new_from_env() -> Result<Self, StoreError> {
        dotenvy::dotenv().ok();
        let url = std::env::var("DATABASE_URL").map_err(|_| {
            StoreError::db_error(
                DatabaseError::ConfigurationError,
                "env var DATABASE_URL is not set",
            )
        })?;
        Self::new(&url)
    }

    pub fn migrate(&mut self) -> Result<(), StoreError> {
        let mut conn = self.conn()?;
        conn.run_pending_migrations(MIGRATIONS)
            .map_err(|e| StoreError::db_error(DatabaseError::MigrationError, e))?;
        Ok(())
    }

    pub fn revert_migrations(&mut self) -> Result<(), StoreError> {
        let mut conn = self.conn()?;
        conn.revert_all_migrations(MIGRATIONS)
            .map_err(|e| StoreError::db_error(DatabaseError::MigrationError, e))?;
        Ok(())
    }

    pub fn reset_db(&mut self) -> Result<(), StoreError> {
        self.revert_migrations()?;
        self.migrate()
    }

    pub(crate) fn conn(&self) -> Result<PgPooledConnection, StoreError> {
        self.pool
            .get()
            .map_err(|e| StoreError::db_error(DatabaseError::ConnectionError, e))
    }
}
