use std::collections::{HashMap, HashSet};

use sam_common::address::{AccountId, DeviceAddress, DeviceId};

use crate::{
    managers::{entities::device::Device, traits::device_manager::DeviceManager},
    ServerError,
};

pub struct InMemoryDeviceManager {
    devices: HashMap<DeviceAddress, Device>,
    account_devices: HashMap<AccountId, HashSet<DeviceAddress>>,
    link_secret: String,
}

impl InMemoryDeviceManager {
    pub fn new(link_secret: String) -> Self {
        InMemoryDeviceManager {
            devices: HashMap::new(),
            account_devices: HashMap::new(),
            link_secret,
        }
    }
}

#[async_trait::async_trait]
impl DeviceManager for InMemoryDeviceManager {
    async fn get_device(&self, account_id: AccountId, id: DeviceId) -> Result<Device, ServerError> {
        let key = DeviceAddress::new(account_id, id);
        self.devices
            .get(&key)
            .ok_or(ServerError::DeviceNotExist)
            .cloned()
    }

    async fn get_devices(&self, account_id: AccountId) -> Result<Vec<DeviceId>, ServerError> {
        let mut devices = vec![];
        if let Some(keys) = self.account_devices.get(&account_id) {
            for key in keys {
                let id = self
                    .devices
                    .get(&key)
                    .ok_or(ServerError::DeviceNotExist)
                    .map(|d| d.id())?;
                devices.push(id);
            }
        } else {
            return Err(ServerError::AccountNotExist);
        }
        Ok(devices)
    }

    async fn next_device_id(&self, account_id: AccountId) -> Result<DeviceId, ServerError> {
        let mut devices = self.get_devices(account_id).await?;
        devices.sort();
        for (i, &num) in devices.iter().enumerate() {
            if num != ((i as u32) + 1).into() {
                return Ok(((i as u32) + 1).into());
            }
        }
        Ok((devices.len() as u32 + 1).into())
    }

    async fn link_secret(&self) -> Result<String, ServerError> {
        Ok(self.link_secret.clone())
    }

    async fn add_device(
        &mut self,
        account_id: AccountId,
        device: Device,
    ) -> Result<(), ServerError> {
        let key = DeviceAddress::new(account_id, device.id());

        if self.devices.contains_key(&key) {
            return Err(ServerError::DeviceExists);
        }

        if !self.account_devices.contains_key(&account_id) {
            self.account_devices.insert(account_id, HashSet::new());
        }

        self.devices.insert(key.clone(), device);

        if let Some(x) = self.account_devices.get_mut(&account_id) {
            x.insert(key);
        }
        Ok(())
    }

    async fn remove_device(
        &mut self,
        account_id: AccountId,
        device_id: DeviceId,
    ) -> Result<(), ServerError> {
        let key = DeviceAddress::new(account_id, device_id);

        if let Some(x) = self.account_devices.get_mut(&account_id) {
            x.remove(&key);
        }

        self.devices
            .remove(&key)
            .ok_or(ServerError::AccountNotExist)
            .map(|_| ())
    }
}
