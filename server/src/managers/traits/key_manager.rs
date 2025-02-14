use libsignal_protocol::IdentityKey;
use sam_common::api::keys::{EcPreKey, PqPreKey, SignedEcPreKey};
use uuid::Uuid;

use crate::ServerError;

#[async_trait::async_trait]
pub trait PreKeyManager: Send {
    async fn get_pre_key(
        &self,
        account_id: &Uuid,
        device_id: &u32,
    ) -> Result<Option<EcPreKey>, ServerError>;
    async fn get_pre_keys(
        &self,
        account_id: &Uuid,
        device_id: &u32,
    ) -> Result<Vec<u32>, ServerError>;
    async fn add_pre_key(
        &mut self,
        account_id: &Uuid,
        device_id: &u32,
        key: EcPreKey,
    ) -> Result<(), ServerError>;
    async fn remove_pre_key(
        &mut self,
        account_id: &Uuid,
        device_id: &u32,
        id: u32,
    ) -> Result<(), ServerError>;
}

#[async_trait::async_trait]
pub trait SignedPreKeyManager {
    async fn get_signed_pre_key(
        &self,
        account_id: &Uuid,
        device_id: &u32,
    ) -> Result<SignedEcPreKey, ServerError>;
    async fn set_signed_pre_key(
        &mut self,
        account_id: &Uuid,
        device_id: &u32,
        identity: &IdentityKey,
        key: SignedEcPreKey,
    ) -> Result<(), ServerError>;
    async fn remove_signed_pre_key(
        &mut self,
        account_id: &Uuid,
        device_id: &u32,
    ) -> Result<(), ServerError>;
}

#[async_trait::async_trait]
pub trait PqPreKeyManager {
    async fn get_pq_pre_key(
        &self,
        account_id: &Uuid,
        device_id: &u32,
    ) -> Result<Option<PqPreKey>, ServerError>;
    async fn get_pq_pre_keys(
        &self,
        account_id: &Uuid,
        device_id: &u32,
    ) -> Result<Vec<u32>, ServerError>;
    async fn add_pq_pre_key(
        &mut self,
        account_id: &Uuid,
        device_id: &u32,
        identity: &IdentityKey,
        key: PqPreKey,
    ) -> Result<(), ServerError>;
    async fn remove_pq_pre_key(
        &mut self,
        account_id: &Uuid,
        device_id: &u32,
        id: u32,
    ) -> Result<(), ServerError>;
}

#[async_trait::async_trait]
pub trait LastResortKeyManager {
    async fn get_last_resort_key(
        &self,
        account_id: &Uuid,
        device_id: &u32,
    ) -> Result<PqPreKey, ServerError>;
    async fn set_last_resort_key(
        &mut self,
        account_id: &Uuid,
        device_id: &u32,
        identity: &IdentityKey,
        key: PqPreKey,
    ) -> Result<(), ServerError>;
    async fn remove_last_resort_key(
        &mut self,
        account_id: &Uuid,
        device_id: &u32,
    ) -> Result<(), ServerError>;
}
