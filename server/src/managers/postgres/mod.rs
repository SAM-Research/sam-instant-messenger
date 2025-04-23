use keys::PostgresKeyManager;
use rand::rngs::OsRng;

use crate::StateType;

mod account;
mod device;
pub mod keys;
mod postgres_connector;

#[cfg(test)]
pub(super) mod test_utils;

pub use account::PostgresAccountManager;
pub use device::PostgresDeviceManager;

pub use postgres_connector::PostgresConnector;

use super::in_memory::message::InMemoryMessageManager;

#[derive(Clone)]
pub struct PostgresStateType;

impl StateType for PostgresStateType {
    type Rng = OsRng;
    type AccountManager = PostgresAccountManager;
    type DeviceManager = PostgresDeviceManager;
    // TODO: Replace with postgres as they are implemented
    type MessageManager = InMemoryMessageManager;
    type KeyManagerType = PostgresKeyManager;
}
