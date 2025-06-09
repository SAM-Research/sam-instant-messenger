use keys::{
    PostgresEcPreKeyManager, PostgresKeyManager, PostgresLastResortPqPreKeyManager,
    PostgresPqPreKeyManager, PostgresSignedPreKeyManager,
};
use rand::rngs::OsRng;
use sqlx::Error;

use crate::{managers::KeyManager, ServerState, StateType};

mod account;
mod device;
pub mod keys;
mod message;
mod postgres_connector;

#[cfg(test)]
pub(super) mod test_utils;

pub use account::PostgresAccountManager;
pub use device::PostgresDeviceManager;
pub use message::PostgresMessageManager;

pub use postgres_connector::PostgresConnector;

#[derive(Clone)]
pub struct PostgresStateType;

impl StateType for PostgresStateType {
    type Rng = OsRng;
    type AccountManager = PostgresAccountManager;
    type DeviceManager = PostgresDeviceManager;
    type KeyManagerType = PostgresKeyManager;
    type MessageManager = PostgresMessageManager;
}

impl ServerState<PostgresStateType> {
    pub async fn connect(url: &str, channel_buffer: usize) -> Result<Self, Error> {
        let conn = PostgresConnector::connect(url).await?;
        let account_mgr = PostgresAccountManager::new(conn.pool());
        let device_mgr = PostgresDeviceManager::new(conn.pool());
        let key_mgr = KeyManager::new(
            PostgresEcPreKeyManager::new(conn.pool()),
            PostgresPqPreKeyManager::new(conn.pool()),
            PostgresSignedPreKeyManager::new(conn.pool()),
            PostgresLastResortPqPreKeyManager::new(conn.pool()),
        );
        let message_mgr = PostgresMessageManager::new(conn.pool(), channel_buffer);

        Ok(Self {
            rng: OsRng,
            accounts: account_mgr,
            devices: device_mgr,
            keys: key_mgr,
            messages: message_mgr,
        })
    }
}
