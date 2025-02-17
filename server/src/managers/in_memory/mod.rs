use device::InMemoryDeviceManager;
use keys::InMemoryKeyManager;
use message::InMemoryMessageManager;
use uuid::Uuid;

pub mod account;
pub mod device;
pub mod keys;
pub mod message;

#[cfg(test)]
pub mod test_utils;

use account::InMemoryAccountManager;

use crate::state::{state_type::StateType, ServerState};

pub(crate) fn device_key(accunt_id: Uuid, id: u32) -> String {
    format!("{}.{}", accunt_id, id)
}

pub struct InMemStateType;

impl StateType for InMemStateType {
    type AccountManager = InMemoryAccountManager;
    type DeviceManager = InMemoryDeviceManager;
    type MessageManager = InMemoryMessageManager;
    type KeyManager = InMemoryKeyManager;
}

impl ServerState<InMemStateType> {
    pub fn in_memory_default(link_secret: String) -> Self {
        ServerState::new(
            InMemoryAccountManager::new(),
            InMemoryDeviceManager::new(link_secret),
            InMemoryMessageManager::new(),
            InMemoryKeyManager::new(),
        )
    }
}
