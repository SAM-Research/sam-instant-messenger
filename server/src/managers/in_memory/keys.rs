use libsignal_protocol::IdentityKey;
use sam_common::api::keys::{Key, SignedKey};
use uuid::Uuid;

use crate::{managers::traits::key_manager::KeyManager, ServerError};

pub struct InMemoryKeyManager {}

#[async_trait::async_trait]
impl KeyManager for InMemoryKeyManager {
    async fn get_key<T: Key>(&self, account_id: &Uuid, device_id: &u32) -> Result<T, ServerError> {
        // Implement the method logic here
        unimplemented!()
    }

    async fn add_key<T: Key>(
        &mut self,
        account_id: &Uuid,
        device_id: &u32,
        key: T,
    ) -> Result<(), ServerError> {
        // Implement the method logic here
        unimplemented!()
    }

    async fn add_signed_key<T: SignedKey>(
        &mut self,
        account_id: &Uuid,
        device_id: &u32,
        identity: &IdentityKey,
        key: T,
    ) -> Result<(), ServerError> {
        // Implement the method logic here
        unimplemented!()
    }

    async fn remove_key<T>(
        &mut self,
        account_id: &Uuid,
        device_id: &u32,
        key: &T,
    ) -> Result<(), ServerError> {
        // Implement the method logic here
        unimplemented!()
    }

    async fn remove_account_keys(&mut self, account_id: &Uuid) -> Result<(), ServerError> {
        // Implement the method logic here
        unimplemented!()
    }
}
