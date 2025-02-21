use device::InMemoryDeviceManager;
use keys::InMemoryKeyManager;
use message::InMemoryMessageManager;

pub mod account;
pub mod device;
pub mod keys;
pub mod message;

#[cfg(test)]
pub mod test_utils;

use account::InMemoryAccountManager;

use crate::state::{state_type::StateType, ServerState};

#[derive(Clone)]
pub struct InMemStateType;

impl StateType for InMemStateType {
    type AccountManager = InMemoryAccountManager;
    type DeviceManager = InMemoryDeviceManager;
    type MessageManager = InMemoryMessageManager;
    type KeyManager = InMemoryKeyManager;
}

impl ServerState<InMemStateType> {
    pub fn in_memory(link_secret: String, message_buffer: usize) -> Self {
        ServerState::new(
            InMemoryAccountManager::default(),
            InMemoryDeviceManager::new(link_secret),
            InMemoryMessageManager::new(message_buffer),
            InMemoryKeyManager::default(),
        )
    }

    #[cfg(test)]
    pub fn in_memory_test() -> Self {
        use test_utils::LINK_SECRET;

        ServerState::new(
            InMemoryAccountManager::default(),
            InMemoryDeviceManager::new(LINK_SECRET.to_string()),
            InMemoryMessageManager::default(),
            InMemoryKeyManager::default(),
        )
    }
}
