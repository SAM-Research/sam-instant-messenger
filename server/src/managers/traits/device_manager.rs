use async_trait::async_trait;
use sam_common::address::{AccountId, DeviceId};

use crate::managers::{entities::device::Device, error::DeviceManagerError};

#[async_trait]
pub trait DeviceManager: Send + Sync + Clone {
    async fn get_device(
        &self,
        account_id: AccountId,
        id: DeviceId,
    ) -> Result<Device, DeviceManagerError>;
    async fn get_devices(&self, account_id: AccountId)
        -> Result<Vec<DeviceId>, DeviceManagerError>;
    async fn next_device_id(&self, account_id: AccountId) -> Result<DeviceId, DeviceManagerError>;
    async fn link_secret(&self) -> Result<String, DeviceManagerError>;
    async fn provision_expire_seconds(&self) -> Result<u32, DeviceManagerError>;
    async fn add_device(
        &mut self,
        account_id: AccountId,
        device: &Device,
    ) -> Result<(), DeviceManagerError>;
    async fn remove_device(
        &mut self,
        account_id: AccountId,
        device_id: DeviceId,
    ) -> Result<(), DeviceManagerError>;
}
