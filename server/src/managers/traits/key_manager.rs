use async_trait::async_trait;
use libsignal_protocol::IdentityKey;
use sam_common::{
    address::{AccountId, DeviceId},
    api::keys::{EcPreKey, PqPreKey, SignedEcPreKey},
};

use crate::managers::error::KeyManagerError;

#[async_trait]
pub trait EcPreKeyManager: Send + Sync + Clone {
    async fn get_pre_key(
        &self,
        account_id: AccountId,
        device_id: DeviceId,
    ) -> Result<Option<EcPreKey>, KeyManagerError>;
    async fn get_pre_key_ids(
        &self,
        account_id: AccountId,
        device_id: DeviceId,
    ) -> Result<Option<Vec<u32>>, KeyManagerError>;
    async fn add_pre_key(
        &mut self,
        account_id: AccountId,
        device_id: DeviceId,
        key: EcPreKey,
    ) -> Result<(), KeyManagerError>;
    async fn remove_pre_key(
        &mut self,
        account_id: AccountId,
        device_id: DeviceId,
        id: u32,
    ) -> Result<(), KeyManagerError>;
}

#[async_trait]
pub trait SignedPreKeyManager: Send + Sync + Clone {
    async fn get_signed_pre_key(
        &self,
        account_id: AccountId,
        device_id: DeviceId,
    ) -> Result<SignedEcPreKey, KeyManagerError>;
    async fn set_signed_pre_key(
        &mut self,
        account_id: AccountId,
        device_id: DeviceId,
        identity: &IdentityKey,
        key: SignedEcPreKey,
    ) -> Result<(), KeyManagerError>;
    async fn remove_signed_pre_key(
        &mut self,
        account_id: AccountId,
        device_id: DeviceId,
    ) -> Result<(), KeyManagerError>;
}

#[async_trait]
pub trait PqPreKeyManager: Send + Sync + Clone {
    async fn get_pq_pre_key(
        &self,
        account_id: AccountId,
        device_id: DeviceId,
    ) -> Result<Option<PqPreKey>, KeyManagerError>;
    async fn get_pq_pre_key_ids(
        &self,
        account_id: AccountId,
        device_id: DeviceId,
    ) -> Result<Option<Vec<u32>>, KeyManagerError>;
    async fn add_pq_pre_key(
        &mut self,
        account_id: AccountId,
        device_id: DeviceId,
        identity: &IdentityKey,
        key: PqPreKey,
    ) -> Result<(), KeyManagerError>;
    async fn remove_pq_pre_key(
        &mut self,
        account_id: AccountId,
        device_id: DeviceId,
        id: u32,
    ) -> Result<(), KeyManagerError>;
}

#[async_trait]
pub trait LastResortPqPreKeyManager: Send + Sync + Clone {
    async fn get_last_resort_key(
        &self,
        account_id: AccountId,
        device_id: DeviceId,
    ) -> Result<PqPreKey, KeyManagerError>;
    async fn set_last_resort_key(
        &mut self,
        account_id: AccountId,
        device_id: DeviceId,
        identity: &IdentityKey,
        key: PqPreKey,
    ) -> Result<(), KeyManagerError>;
    async fn remove_last_resort_key(
        &mut self,
        account_id: AccountId,
        device_id: DeviceId,
    ) -> Result<(), KeyManagerError>;
}
