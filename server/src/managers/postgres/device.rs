use sam_common::{AccountId, DeviceId};

use crate::{
    managers::{entities::Device, traits::device_manager::DeviceManager},
    ServerError,
};

#[derive(Debug, Clone)]
pub struct PostgresDeviceManager {}

#[async_trait::async_trait]
impl DeviceManager for PostgresDeviceManager {
    async fn get_device(
        &self,
        _account_id: AccountId,
        _id: DeviceId,
    ) -> Result<Device, ServerError> {
        todo!()
    }

    async fn get_devices(&self, _account_id: AccountId) -> Result<Vec<DeviceId>, ServerError> {
        todo!()
    }

    async fn next_device_id(&self, _account_id: AccountId) -> Result<DeviceId, ServerError> {
        todo!()
    }

    async fn link_secret(&self) -> Result<String, ServerError> {
        todo!()
    }

    async fn provision_expire_seconds(&self) -> Result<u64, ServerError> {
        todo!()
    }

    async fn add_device(
        &mut self,
        _account_id: AccountId,
        _device: &Device,
    ) -> Result<(), ServerError> {
        todo!()
    }

    async fn remove_device(
        &mut self,
        _account_id: AccountId,
        _device_id: DeviceId,
    ) -> Result<(), ServerError> {
        todo!()
    }
}
