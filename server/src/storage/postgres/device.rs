use async_trait::async_trait;
use sam_common::address::{AccountId, DeviceAddress};

use crate::storage::{device::Device, error::DeviceStoreError, traits::DeviceStore};

#[derive(Debug)]
pub struct PostgresDeviceStore {}

#[async_trait(?Send)]
impl DeviceStore for PostgresDeviceStore {
    async fn add_device(
        &mut self,
        _device: Device,
        _service_id: &AccountId,
    ) -> Result<(), DeviceStoreError> {
        todo!()
    }

    async fn get_all_devices(
        &self,
        _service_id: &AccountId,
    ) -> Result<Vec<Device>, DeviceStoreError> {
        todo!()
    }

    async fn get_device(&self, _address: &DeviceAddress) -> Result<Device, DeviceStoreError> {
        todo!()
    }

    async fn delete_device(&mut self, _address: &DeviceAddress) -> Result<(), DeviceStoreError> {
        todo!()
    }
}
