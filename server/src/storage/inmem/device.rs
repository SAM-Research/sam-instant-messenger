use std::collections::HashMap;

use async_trait::async_trait;
use sam_common::address::{AccountId, DeviceAddress, DeviceId};

use crate::storage::device::Device;
use crate::storage::error::DeviceStoreError;
use crate::storage::traits::DeviceStore;

type DeviceMap = HashMap<DeviceId, Device>;

#[derive(Debug)]
pub struct PostgresDeviceStore {
    devices: HashMap<AccountId, DeviceMap>,
}

#[async_trait(?Send)]
impl DeviceStore for PostgresDeviceStore {
    async fn add_device(
        &mut self,
        device: Device,
        service_id: &AccountId,
    ) -> Result<(), DeviceStoreError> {
        let entry = self.devices.entry(service_id.to_owned()).or_default();

        if entry.contains_key(&device.id()) {
            return Err(DeviceStoreError::DeviceIdTaken(device.id()));
        }
        entry.insert(device.id(), device.to_owned());
        Ok(())
    }

    async fn get_all_devices(
        &self,
        service_id: &AccountId,
    ) -> Result<Vec<Device>, DeviceStoreError> {
        Ok(self
            .devices
            .get(service_id)
            .ok_or(DeviceStoreError::AccountNotFound(service_id.to_owned()))?
            .values()
            .cloned()
            .collect())
    }

    async fn get_device(&self, address: &DeviceAddress) -> Result<Device, DeviceStoreError> {
        Ok(self
            .devices
            .get(&address.account_id())
            .ok_or(DeviceStoreError::AccountNotFound(
                address.account_id().to_owned(),
            ))?
            .get(&address.device_id())
            .ok_or(DeviceStoreError::DeviceNotFound(address.device_id()))?
            .to_owned())
    }

    async fn delete_device(&mut self, address: &DeviceAddress) -> Result<(), DeviceStoreError> {
        Ok(self
            .devices
            .get_mut(&address.account_id())
            .ok_or(DeviceStoreError::AccountNotFound(
                address.account_id().to_owned(),
            ))?
            .remove(&address.device_id())
            .ok_or(DeviceStoreError::DeviceNotFound(address.device_id()))
            .map(|_| ())?)
    }
}
