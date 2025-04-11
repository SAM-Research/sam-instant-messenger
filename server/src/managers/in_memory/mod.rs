use device::InMemoryDeviceManager;
use keys::{
    InMemoryEcPreKeyManager, InMemoryKeyManager, InMemoryLastResortPqPreKeyManager,
    InMemoryPqPreKeyManager, InMemorySignedPreKeyManager,
};
use message::InMemoryMessageManager;

pub mod account;
pub mod device;
pub mod keys;
pub mod message;

#[cfg(test)]
pub mod test_utils;

use account::InMemoryAccountManager;

use crate::state::{state_type::StateType, ServerState};

use super::KeyManager;

#[derive(Clone)]
pub struct InMemStateType;

impl StateType for InMemStateType {
    type AccountManager = InMemoryAccountManager;
    type DeviceManager = InMemoryDeviceManager;
    type MessageManager = InMemoryMessageManager;
    type KeyManagerType = InMemoryKeyManager;
}

impl ServerState<InMemStateType> {
    pub fn in_memory(
        link_secret: String,
        provision_expire_seconds: u32, // signal uses 600 seconds
        message_buffer: usize,
    ) -> Self {
        ServerState::new(
            InMemoryAccountManager::default(),
            InMemoryDeviceManager::new(link_secret, provision_expire_seconds),
            InMemoryMessageManager::new(message_buffer),
            KeyManager::new(
                InMemoryEcPreKeyManager::default(),
                InMemoryPqPreKeyManager::default(),
                InMemorySignedPreKeyManager::default(),
                InMemoryLastResortPqPreKeyManager::default(),
            ),
        )
    }

    #[cfg(test)]
    pub fn in_memory_test() -> Self {
        use test_utils::LINK_SECRET;

        ServerState::new(
            InMemoryAccountManager::default(),
            InMemoryDeviceManager::new(LINK_SECRET.to_string(), 600),
            InMemoryMessageManager::default(),
            KeyManager::new(
                InMemoryEcPreKeyManager::default(),
                InMemoryPqPreKeyManager::default(),
                InMemorySignedPreKeyManager::default(),
                InMemoryLastResortPqPreKeyManager::default(),
            ),
        )
    }
}
