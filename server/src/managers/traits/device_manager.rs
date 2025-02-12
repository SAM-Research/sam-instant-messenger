use uuid::Uuid;

use crate::{managers::entities::device::Device, ServerError};

#[async_trait::async_trait]
pub trait DeviceManager: Send {
    async fn get_device(&self, account_id: &Uuid, id: &u32) -> Result<Device, ServerError>;
    async fn get_devices(&self, account_id: &Uuid) -> Result<Vec<u32>, ServerError>;
    async fn next_device_id(&self, account_id: &Uuid) -> Result<u32, ServerError>;
    async fn link_secret(&self) -> Result<String, ServerError>;
    async fn add_device(&mut self, account_id: &Uuid, device: Device) -> Result<(), ServerError>;
    async fn remove_device(&mut self, account_id: &Uuid, device_id: u32)
        -> Result<(), ServerError>;
}
