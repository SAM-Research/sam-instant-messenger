use rstest::fixture;
use sqlx::{Pool, Postgres};

use crate::managers::KeyManager;

use super::{
    account::PostgresAccountManager,
    device::PostgresDeviceManager,
    keys::{
        PostgresEcPreKeyManager, PostgresKeyManager, PostgresLastResortPqPreKeyManager,
        PostgresPqPreKeyManager, PostgresSignedPreKeyManager,
    },
};

#[fixture]
pub fn connection_str() -> &'static str {
    "postgres://test:test@127.0.0.1:5432/sam_test_db"
}

#[fixture]
pub async fn accounts(connection_str: &str) -> PostgresAccountManager {
    let pool = Pool::<Postgres>::connect(connection_str)
        .await
        .expect("Can connect to postgres");

    PostgresAccountManager::new(pool)
}

#[fixture]
pub async fn devices(connection_str: &str) -> PostgresDeviceManager {
    let pool = Pool::<Postgres>::connect(connection_str)
        .await
        .expect("Can connect to postgres");

    PostgresDeviceManager::new(pool)
}

#[fixture]
pub async fn keys(connection_str: &str) -> KeyManager<PostgresKeyManager> {
    let pool = Pool::<Postgres>::connect(connection_str)
        .await
        .expect("Can connect to postgres");

    KeyManager::new(
        PostgresEcPreKeyManager::new(pool.clone()),
        PostgresPqPreKeyManager::new(pool.clone()),
        PostgresSignedPreKeyManager::new(pool.clone()),
        PostgresLastResortPqPreKeyManager::new(pool),
    )
}
