use async_trait::async_trait;
use libsignal_protocol::{Aci, ProtocolAddress};

use crate::storage::{error::DeviceStoreError, Device, DeviceCapability, DeviceStore};

#[derive(Debug)]
pub struct PostgresDeviceStore {}

#[async_trait(?Send)]
impl DeviceStore for PostgresDeviceStore {
    async fn add_device(
        &mut self,
        _service_id: &Aci,
        _device: &Device,
    ) -> Result<(), DeviceStoreError> {
        todo!()
    }

    async fn get_all_devices(&self, _service_id: &Aci) -> Result<Vec<Device>, DeviceStoreError> {
        todo!()
    }

    async fn get_device(&self, _address: &ProtocolAddress) -> Result<Device, DeviceStoreError> {
        todo!()
    }

    async fn delete_device(&mut self, _address: &ProtocolAddress) -> Result<(), DeviceStoreError> {
        todo!()
    }

    async fn get_device_capabilities(
        &self,
        _address: &ProtocolAddress,
    ) -> Result<Vec<DeviceCapability>, DeviceStoreError> {
        todo!()
    }

    async fn get_all_device_capabilities(
        &self,
        _service_id: &Aci,
    ) -> Result<Vec<(i32, DeviceCapability)>, DeviceStoreError> {
        todo!()
    }

    async fn add_used_device_link_token(
        &mut self,
        _device_link_token: String,
    ) -> Result<(), DeviceStoreError> {
        todo!()
    }
}
