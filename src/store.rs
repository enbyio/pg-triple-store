use std::time::Duration;

use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use diesel::PgConnection;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

use crate::error::{DatabaseError, StoreError};

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

pub(crate) type PgPool = Pool<ConnectionManager<PgConnection>>;
pub(crate) type PgPooledConnection = PooledConnection<ConnectionManager<PgConnection>>;

/// Centerpiece of this library. This processes sparql queries / other functions.
pub struct TripleStore {
    pool: PgPool,
}

/// Allows for selecting whether to target objects or predicates in api functions.
pub enum ElementType {
    Object,
    Predicate,
}

impl TripleStore {
    /// Creates database connection from url of format:
    /// postgres://postgres:yourpassword@localhost:5432/triple_store \
    /// Checks if all tables exist correctly (and if not applies the required migrations)
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

    /// Creates database connection from url of format in env var **$DATABASE_URL**:
    /// postgres://postgres:yourpassword@localhost:5432/triple_store \
    /// Checks if all tables exist correctly (and if not applies the required migrations)
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

    /// Manually run database migrations.
    /// This is non-destructive and only ensures all migrations are correctly run.
    pub fn migrate(&mut self) -> Result<(), StoreError> {
        let mut conn = self.conn()?;
        conn.run_pending_migrations(MIGRATIONS)
            .map_err(|e| StoreError::db_error(DatabaseError::MigrationError, e))?;
        Ok(())
    }

    fn revert_migrations(&mut self) -> Result<(), StoreError> {
        let mut conn = self.conn()?;
        conn.revert_all_migrations(MIGRATIONS)
            .map_err(|e| StoreError::db_error(DatabaseError::MigrationError, e))?;
        Ok(())
    }
    /// **Warning**: this will wipe the data in your db*
    /// Revert migrations to delete existing tables and then apply migrations again.
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
