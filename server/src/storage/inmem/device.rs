use std::collections::HashMap;

use async_trait::async_trait;
use libsignal_protocol::{Aci, ProtocolAddress};

use crate::storage::error::DeviceStoreError;
use crate::storage::{Device, DeviceCapability, DeviceStore};

type DeviceMap = HashMap<u32, Device>;

#[derive(Debug)]
pub struct PostgresDeviceStore {
    _devices: HashMap<Aci, DeviceMap>,
}

#[async_trait(?Send)]
impl DeviceStore for PostgresDeviceStore {
    async fn add_device(
        &mut self,
        _service_id: &Aci,
        _device: &Device,
    ) -> Result<(), DeviceStoreError> {
        let entry = self._devices.entry(*_service_id).or_insert(HashMap::new());

        if entry.contains_key(&_device.id) {
            return Err(DeviceStoreError::DeviceIdTaken(_device.id));
        }
        entry.insert(_device.id, _device.to_owned());
        Ok(())
    }

    async fn get_all_devices(&self, _service_id: &Aci) -> Result<Vec<Device>, DeviceStoreError> {
        Ok(self
            ._devices
            .get(_service_id)
            .ok_or(DeviceStoreError::AccountNotFound(
                _service_id.service_id_string(),
            ))?
            .values()
            .cloned()
            .collect())
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
