use sam_common::address::{AccountId, DeviceId};

use crate::{managers::entities::device::Device, ServerError};

#[async_trait::async_trait]
pub trait DeviceManager: Send + Sync + Clone {
    async fn get_device(&self, account_id: AccountId, id: DeviceId) -> Result<Device, ServerError>;
    async fn get_devices(&self, account_id: AccountId) -> Result<Vec<DeviceId>, ServerError>;
    async fn next_device_id(&self, account_id: AccountId) -> Result<DeviceId, ServerError>;
    async fn link_secret(&self) -> Result<String, ServerError>;
    async fn add_device(
        &mut self,
        account_id: AccountId,
        device: &Device,
    ) -> Result<(), ServerError>;
    async fn remove_device(
        &mut self,
        account_id: AccountId,
        device_id: DeviceId,
    ) -> Result<(), ServerError>;
}
