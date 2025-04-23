use sqlx::{Pool, Postgres};

use super::{account::PostgresAccountManager, device::PostgresDeviceManager};

pub fn connection_str() -> &'static str {
    "postgres://test:test@127.0.0.1:5432/sam_test_db"
}

pub async fn accounts(connection_str: &str) -> PostgresAccountManager {
    let pool = Pool::<Postgres>::connect(connection_str)
        .await
        .expect("Can connect to postgres");

    PostgresAccountManager::new(pool)
}

pub async fn devices(connection_str: &str) -> PostgresDeviceManager {
    let pool = Pool::<Postgres>::connect(connection_str)
        .await
        .expect("Can connect to postgres");

    PostgresDeviceManager::new(pool)
}
