use std::collections::{HashMap, HashSet};

use uuid::Uuid;

use crate::{
    managers::{entities::device::Device, traits::device_manager::DeviceManager},
    ServerError,
};

use super::device_key;

pub struct InMemoryDeviceManager {
    devices: HashMap<String, Device>,
    account_devices: HashMap<Uuid, HashSet<String>>,
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
    async fn get_device(&self, account_id: Uuid, id: u32) -> Result<Device, ServerError> {
        let key = device_key(account_id, id);
        self.devices
            .get(&key)
            .ok_or(ServerError::DeviceNotExist)
            .cloned()
    }

    async fn get_devices(&self, account_id: Uuid) -> Result<Vec<u32>, ServerError> {
        let mut devices = vec![];
        if let Some(keys) = self.account_devices.get(&account_id) {
            for key in keys {
                let id = self
                    .devices
                    .get(key)
                    .ok_or(ServerError::DeviceNotExist)
                    .map(|d| d.id())?;
                devices.push(id);
            }
        } else {
            return Err(ServerError::AccountNotExist);
        }
        Ok(devices)
    }

    async fn next_device_id(&self, account_id: Uuid) -> Result<u32, ServerError> {
        let mut devices = self.get_devices(account_id).await?;
        devices.sort();
        for (i, &num) in devices.iter().enumerate() {
            if num != (i as u32) + 1 {
                return Ok((i as u32) + 1);
            }
        }
        Ok(devices.len() as u32 + 1)
    }

    async fn link_secret(&self) -> Result<String, ServerError> {
        Ok(self.link_secret.clone())
    }

    async fn add_device(&mut self, account_id: Uuid, device: &Device) -> Result<(), ServerError> {
        let key = device_key(account_id, device.id());

        if self.devices.contains_key(&key) {
            return Err(ServerError::DeviceExists);
        }

        if !self.account_devices.contains_key(&account_id) {
            self.account_devices.insert(account_id, HashSet::new());
        }

        self.devices.insert(key.clone(), device.clone());

        if let Some(x) = self.account_devices.get_mut(&account_id) {
            x.insert(key);
        }
        Ok(())
    }

    async fn remove_device(&mut self, account_id: Uuid, device_id: u32) -> Result<(), ServerError> {
        let key = device_key(account_id, device_id);

        if let Some(x) = self.account_devices.get_mut(&account_id) {
            x.remove(&key);
        }

        self.devices
            .remove(&key)
            .ok_or(ServerError::AccountNotExist)
            .map(|_| ())
    }
}
