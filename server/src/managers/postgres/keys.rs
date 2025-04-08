use async_trait::async_trait;
use libsignal_protocol::IdentityKey;
use sam_common::{
    api::{EcPreKey, PqPreKey, SignedEcPreKey},
    AccountId, DeviceId,
};
use sqlx::{Pool, Postgres};

use crate::managers::{
    error::KeyManagerError,
    traits::key_manager::{
        EcPreKeyManager, LastResortPqPreKeyManager, PqPreKeyManager, SignedPreKeyManager,
    },
    KeyManagerType,
};

#[derive(Clone)]
pub struct PostgresKeyManager;

impl KeyManagerType for PostgresKeyManager {
    type EcPreKeyManager = PostgresEcPreKeyManager;

    type PqPreKeyManager = PostgresPqPreKeyManager;

    type SignedPreKeyManager = PostgresSignedPreKeyManager;

    type LastResortPqPreKeyManager = PostgresLastResortPqPreKeyManager;
}

#[derive(Debug, Clone)]
pub struct PostgresEcPreKeyManager {
    pool: Pool<Postgres>,
}

impl PostgresEcPreKeyManager {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl EcPreKeyManager for PostgresEcPreKeyManager {
    async fn get_pre_key(
        &self,
        _account_id: AccountId,
        _device_id: DeviceId,
    ) -> Result<Option<EcPreKey>, KeyManagerError> {
        todo!()
    }

    async fn get_pre_key_ids(
        &self,
        _account_id: AccountId,
        _device_id: DeviceId,
    ) -> Result<Option<Vec<u32>>, KeyManagerError> {
        todo!()
    }

    async fn add_pre_key(
        &mut self,
        _account_id: AccountId,
        _device_id: DeviceId,
        _key: EcPreKey,
    ) -> Result<(), KeyManagerError> {
        todo!()
    }

    async fn remove_pre_key(
        &mut self,
        _account_id: AccountId,
        _device_id: DeviceId,
        _id: u32,
    ) -> Result<(), KeyManagerError> {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub struct PostgresSignedPreKeyManager {
    pool: Pool<Postgres>,
}

impl PostgresSignedPreKeyManager {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SignedPreKeyManager for PostgresSignedPreKeyManager {
    async fn get_signed_pre_key(
        &self,
        _account_id: AccountId,
        _device_id: DeviceId,
    ) -> Result<SignedEcPreKey, KeyManagerError> {
        todo!()
    }

    async fn set_signed_pre_key(
        &mut self,
        _account_id: AccountId,
        _device_id: DeviceId,
        _identity: &IdentityKey,
        _key: SignedEcPreKey,
    ) -> Result<(), KeyManagerError> {
        todo!()
    }

    async fn remove_signed_pre_key(
        &mut self,
        _account_id: AccountId,
        _device_id: DeviceId,
    ) -> Result<(), KeyManagerError> {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub struct PostgresPqPreKeyManager {
    pool: Pool<Postgres>,
}

impl PostgresPqPreKeyManager {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PqPreKeyManager for PostgresPqPreKeyManager {
    async fn get_pq_pre_key(
        &self,
        _account_id: AccountId,
        _device_id: DeviceId,
    ) -> Result<Option<PqPreKey>, KeyManagerError> {
        todo!()
    }

    async fn get_pq_pre_key_ids(
        &self,
        _account_id: AccountId,
        _device_id: DeviceId,
    ) -> Result<Option<Vec<u32>>, KeyManagerError> {
        todo!()
    }

    async fn add_pq_pre_key(
        &mut self,
        _account_id: AccountId,
        _device_id: DeviceId,
        _identity: &IdentityKey,
        _key: PqPreKey,
    ) -> Result<(), KeyManagerError> {
        todo!()
    }

    async fn remove_pq_pre_key(
        &mut self,
        _account_id: AccountId,
        _device_id: DeviceId,
        _id: u32,
    ) -> Result<(), KeyManagerError> {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub struct PostgresLastResortPqPreKeyManager {
    pool: Pool<Postgres>,
}

impl PostgresLastResortPqPreKeyManager {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl LastResortPqPreKeyManager for PostgresLastResortPqPreKeyManager {
    async fn get_last_resort_key(
        &self,
        _account_id: AccountId,
        _device_id: DeviceId,
    ) -> Result<PqPreKey, KeyManagerError> {
        todo!()
    }

    async fn set_last_resort_key(
        &mut self,
        _account_id: AccountId,
        _device_id: DeviceId,
        _identity: &IdentityKey,
        _key: PqPreKey,
    ) -> Result<(), KeyManagerError> {
        todo!()
    }

    async fn remove_last_resort_key(
        &mut self,
        _account_id: AccountId,
        _device_id: DeviceId,
    ) -> Result<(), KeyManagerError> {
        todo!()
    }
}
