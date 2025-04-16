use log::debug;
use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};

use crate::storage::error::DatabaseError;

#[derive(Clone)]
pub struct SqliteConnector {
    pool: Pool<Sqlite>,
}

impl SqliteConnector {
    pub async fn connect(url: &str) -> Result<Self, DatabaseError> {
        Ok(Self {
            pool: connect(url).await?,
        })
    }

    pub async fn migrate(url: &str) -> Result<Self, DatabaseError> {
        let pool = connect(url).await?;
        sqlx::migrate!("database/migrations")
            .run(&pool)
            .await
            .inspect_err(|e| debug!("{e}"))
            .map_err(|_| DatabaseError::FailedToMigrate)?;
        Ok(Self { pool })
    }

    pub fn pool(&self) -> Pool<Sqlite> {
        self.pool.clone()
    }
}

impl From<SqliteConnector> for Pool<Sqlite> {
    fn from(val: SqliteConnector) -> Self {
        val.pool
    }
}

async fn connect(url: &str) -> Result<Pool<Sqlite>, DatabaseError> {
    SqlitePoolOptions::new()
        .connect(url)
        .await
        .inspect_err(|e| debug!("{e}"))
        .map_err(|_| DatabaseError::FailedToConnect)
}
