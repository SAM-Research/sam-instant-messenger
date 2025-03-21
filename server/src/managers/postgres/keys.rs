use libsignal_protocol::IdentityKey;
use sam_common::{
    api::{EcPreKey, PqPreKey, SignedEcPreKey},
    AccountId, DeviceId,
};

use crate::{
    managers::traits::key_manager::{
        LastResortKeyManager, PqPreKeyManager, PreKeyManager, SignedPreKeyManager,
    },
    ServerError,
};

#[derive(Debug, Clone)]
pub struct PostgresKeyManager {}

#[async_trait::async_trait]
impl PreKeyManager for PostgresKeyManager {
    async fn get_pre_key(
        &self,
        _account_id: AccountId,
        _device_id: DeviceId,
    ) -> Result<Option<EcPreKey>, ServerError> {
        todo!()
    }

    async fn get_pre_key_ids(
        &self,
        _account_id: AccountId,
        _device_id: DeviceId,
    ) -> Result<Option<Vec<u32>>, ServerError> {
        todo!()
    }

    async fn add_pre_key(
        &mut self,
        _account_id: AccountId,
        _device_id: DeviceId,
        _key: EcPreKey,
    ) -> Result<(), ServerError> {
        todo!()
    }

    async fn remove_pre_key(
        &mut self,
        _account_id: AccountId,
        _device_id: DeviceId,
        _id: u32,
    ) -> Result<(), ServerError> {
        todo!()
    }
}

#[async_trait::async_trait]
impl SignedPreKeyManager for PostgresKeyManager {
    async fn get_signed_pre_key(
        &self,
        _account_id: AccountId,
        _device_id: DeviceId,
    ) -> Result<SignedEcPreKey, ServerError> {
        todo!()
    }

    async fn set_signed_pre_key(
        &mut self,
        _account_id: AccountId,
        _device_id: DeviceId,
        _identity: &IdentityKey,
        _key: SignedEcPreKey,
    ) -> Result<(), ServerError> {
        todo!()
    }

    async fn remove_signed_pre_key(
        &mut self,
        _account_id: AccountId,
        _device_id: DeviceId,
    ) -> Result<(), ServerError> {
        todo!()
    }
}

#[async_trait::async_trait]
impl PqPreKeyManager for PostgresKeyManager {
    async fn get_pq_pre_key(
        &self,
        _account_id: AccountId,
        _device_id: DeviceId,
    ) -> Result<Option<PqPreKey>, ServerError> {
        todo!()
    }

    async fn get_pq_pre_key_ids(
        &self,
        _account_id: AccountId,
        _device_id: DeviceId,
    ) -> Result<Option<Vec<u32>>, ServerError> {
        todo!()
    }

    async fn add_pq_pre_key(
        &mut self,
        _account_id: AccountId,
        _device_id: DeviceId,
        _identity: &IdentityKey,
        _key: PqPreKey,
    ) -> Result<(), ServerError> {
        todo!()
    }

    async fn remove_pq_pre_key(
        &mut self,
        _account_id: AccountId,
        _device_id: DeviceId,
        _id: u32,
    ) -> Result<(), ServerError> {
        todo!()
    }
}

#[async_trait::async_trait]
impl LastResortKeyManager for PostgresKeyManager {
    async fn get_last_resort_key(
        &self,
        _account_id: AccountId,
        _device_id: DeviceId,
    ) -> Result<PqPreKey, ServerError> {
        todo!()
    }

    async fn set_last_resort_key(
        &mut self,
        _account_id: AccountId,
        _device_id: DeviceId,
        _identity: &IdentityKey,
        _key: PqPreKey,
    ) -> Result<(), ServerError> {
        todo!()
    }

    async fn remove_last_resort_key(
        &mut self,
        _account_id: AccountId,
        _device_id: DeviceId,
    ) -> Result<(), ServerError> {
        todo!()
    }
}
