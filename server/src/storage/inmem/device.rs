use std::collections::HashMap;

use async_trait::async_trait;
use libsignal_protocol::{Aci, ProtocolAddress};

use crate::storage::error::DeviceStoreError;
use crate::storage::{Device, DeviceStore};

type DeviceMap = HashMap<u32, Device>;

#[derive(Debug)]
pub struct PostgresDeviceStore {
    devices: HashMap<Aci, DeviceMap>,
}

#[async_trait(?Send)]
impl DeviceStore for PostgresDeviceStore {
    async fn add_device(
        &mut self,
        service_id: &Aci,
        device: &Device,
    ) -> Result<(), DeviceStoreError> {
        let entry = self.devices.entry(*service_id).or_insert(HashMap::new());

        if entry.contains_key(&device.id) {
            return Err(DeviceStoreError::DeviceIdTaken(device.id));
        }
        entry.insert(device.id, device.to_owned());
        Ok(())
    }

    async fn get_all_devices(&self, service_id: &Aci) -> Result<Vec<Device>, DeviceStoreError> {
        Ok(self
            .devices
            .get(service_id)
            .ok_or(DeviceStoreError::AccountNotFound(
                service_id.service_id_string(),
            ))?
            .values()
            .cloned()
            .collect())
    }

    async fn get_device(&self, address: &ProtocolAddress) -> Result<Device, DeviceStoreError> {
        let aci = Aci::parse_from_service_id_string(address.name()).unwrap();
        Ok(self
            .devices
            .get(&aci)
            .ok_or(DeviceStoreError::AccountNotFound(address.name().to_owned()))?
            .get(&address.device_id().into())
            .ok_or(DeviceStoreError::DeviceNotFound(address.device_id().into()))?
            .to_owned())
    }

    async fn delete_device(&mut self, address: &ProtocolAddress) -> Result<(), DeviceStoreError> {
        let aci = Aci::parse_from_service_id_string(address.name()).unwrap();
        Ok(self
            .devices
            .get_mut(&aci)
            .ok_or(DeviceStoreError::AccountNotFound(address.name().to_owned()))?
            .remove(&address.device_id().into())
            .ok_or(DeviceStoreError::DeviceNotFound(address.device_id().into()))
            .map(|_| ())?)
    }
}
