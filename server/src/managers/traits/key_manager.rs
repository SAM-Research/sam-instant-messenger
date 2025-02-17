use libsignal_protocol::IdentityKey;
use sam_common::{
    address::{AccountId, DeviceId},
    api::keys::{EcPreKey, PqPreKey, SignedEcPreKey},
};

use crate::ServerError;

#[async_trait::async_trait]
pub trait PreKeyManager: Send {
    async fn get_pre_key(
        &self,
        account_id: AccountId,
        device_id: DeviceId,
    ) -> Result<Option<EcPreKey>, ServerError>;
    async fn get_pre_keys(
        &self,
        account_id: AccountId,
        device_id: DeviceId,
    ) -> Result<Vec<u32>, ServerError>;
    async fn add_pre_key(
        &mut self,
        account_id: AccountId,
        device_id: DeviceId,
        key: EcPreKey,
    ) -> Result<(), ServerError>;
    async fn remove_pre_key(
        &mut self,
        account_id: AccountId,
        device_id: DeviceId,
        id: u32,
    ) -> Result<(), ServerError>;
}

#[async_trait::async_trait]
pub trait SignedPreKeyManager {
    async fn get_signed_pre_key(
        &self,
        account_id: AccountId,
        device_id: DeviceId,
    ) -> Result<SignedEcPreKey, ServerError>;
    async fn set_signed_pre_key(
        &mut self,
        account_id: AccountId,
        device_id: DeviceId,
        identity: &IdentityKey,
        key: SignedEcPreKey,
    ) -> Result<(), ServerError>;
    async fn remove_signed_pre_key(
        &mut self,
        account_id: AccountId,
        device_id: DeviceId,
    ) -> Result<(), ServerError>;
}

#[async_trait::async_trait]
pub trait PqPreKeyManager {
    async fn get_pq_pre_key(
        &self,
        account_id: AccountId,
        device_id: DeviceId,
    ) -> Result<Option<PqPreKey>, ServerError>;
    async fn get_pq_pre_keys(
        &self,
        account_id: AccountId,
        device_id: DeviceId,
    ) -> Result<Vec<u32>, ServerError>;
    async fn add_pq_pre_key(
        &mut self,
        account_id: AccountId,
        device_id: DeviceId,
        identity: &IdentityKey,
        key: PqPreKey,
    ) -> Result<(), ServerError>;
    async fn remove_pq_pre_key(
        &mut self,
        account_id: AccountId,
        device_id: DeviceId,
        id: u32,
    ) -> Result<(), ServerError>;
}

#[async_trait::async_trait]
pub trait LastResortKeyManager {
    async fn get_last_resort_key(
        &self,
        account_id: AccountId,
        device_id: DeviceId,
    ) -> Result<PqPreKey, ServerError>;
    async fn set_last_resort_key(
        &mut self,
        account_id: AccountId,
        device_id: DeviceId,
        identity: &IdentityKey,
        key: PqPreKey,
    ) -> Result<(), ServerError>;
    async fn remove_last_resort_key(
        &mut self,
        account_id: AccountId,
        device_id: DeviceId,
    ) -> Result<(), ServerError>;
}
