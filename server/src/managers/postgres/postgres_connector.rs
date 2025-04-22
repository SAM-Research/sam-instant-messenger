use log::debug;
use sqlx::{error::Error, postgres::PgPoolOptions, Pool, Postgres};

#[derive(Clone)]
pub struct PostgresConnector {
    pool: Pool<Postgres>,
}

impl PostgresConnector {
    pub async fn connect(url: &str) -> Result<Self, Error> {
        Ok(Self {
            pool: connect(url).await?,
        })
    }

    pub fn pool(&self) -> Pool<Postgres> {
        self.pool.clone()
    }
}

impl From<PostgresConnector> for Pool<Postgres> {
    fn from(val: PostgresConnector) -> Self {
        val.pool
    }
}

async fn connect(url: &str) -> Result<Pool<Postgres>, Error> {
    PgPoolOptions::new()
        .connect(url)
        .await
        .inspect_err(|e| debug!("{e}"))
}
