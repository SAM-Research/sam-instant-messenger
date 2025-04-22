use account::PostgresAccountManager;
use rand::rngs::OsRng;

use crate::StateType;

use super::in_memory::{
    device::InMemoryDeviceManager, keys::InMemoryKeyManager, message::InMemoryMessageManager,
};

pub mod account;

#[derive(Clone)]
pub struct PostgresStateType;

impl StateType for PostgresStateType {
    type Rng = OsRng;
    type AccountManager = PostgresAccountManager;
    type DeviceManager = InMemoryDeviceManager;
    type MessageManager = InMemoryMessageManager;
    type KeyManagerType = InMemoryKeyManager;
}
