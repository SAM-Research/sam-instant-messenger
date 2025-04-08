use async_trait::async_trait;
use sam_common::{AccountId, DeviceId};
use sqlx::{Pool, Postgres};

use crate::managers::{
    entities::Device, error::DeviceManagerError, traits::device_manager::DeviceManager,
};

#[derive(Debug, Clone)]
pub struct PostgresDeviceManager {
    pool: Pool<Postgres>,
}

impl PostgresDeviceManager {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DeviceManager for PostgresDeviceManager {
    async fn get_device(
        &self,
        _account_id: AccountId,
        _id: DeviceId,
    ) -> Result<Device, DeviceManagerError> {
        todo!()
    }

    async fn get_devices(
        &self,
        _account_id: AccountId,
    ) -> Result<Vec<DeviceId>, DeviceManagerError> {
        todo!()
    }

    async fn next_device_id(&self, _account_id: AccountId) -> Result<DeviceId, DeviceManagerError> {
        todo!()
    }

    async fn link_secret(&self) -> Result<String, DeviceManagerError> {
        todo!()
    }

    async fn provision_expire_seconds(&self) -> Result<u64, DeviceManagerError> {
        todo!()
    }

    async fn add_device(
        &mut self,
        _account_id: AccountId,
        _device: &Device,
    ) -> Result<(), DeviceManagerError> {
        todo!()
    }

    async fn remove_device(
        &mut self,
        _account_id: AccountId,
        _device_id: DeviceId,
    ) -> Result<(), DeviceManagerError> {
        todo!()
    }
}
