use libsignal_protocol::IdentityKey;
use sam_common::api::keys::{Key, SignedKey};
use uuid::Uuid;

use crate::ServerError;

#[async_trait::async_trait]
pub trait KeyManager: Send {
    async fn get_key<T: Key>(&self, account_id: &Uuid, device_id: &u32) -> Result<T, ServerError>;

    async fn add_key<T: Key>(
        &mut self,
        account_id: &Uuid,
        device_id: &u32,
        key: T,
    ) -> Result<(), ServerError>;
    async fn add_signed_key<T: SignedKey>(
        &mut self,
        account_id: &Uuid,
        device_id: &u32,
        identity: &IdentityKey,
        key: T,
    ) -> Result<(), ServerError>;
    async fn remove_key<T>(
        &mut self,
        account_id: &Uuid,
        device_id: &u32,
        key: &T,
    ) -> Result<(), ServerError>;
    async fn remove_account_keys(&mut self, account_id: &Uuid) -> Result<(), ServerError>;
}
