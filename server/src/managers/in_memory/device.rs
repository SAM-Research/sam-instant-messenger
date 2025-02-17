use sam_common::address::{AccountId, DeviceAddress, DeviceId};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use tokio::sync::Mutex;

use crate::{
    managers::{entities::device::Device, traits::device_manager::DeviceManager},
    ServerError,
};

#[derive(Clone)]
pub struct InMemoryDeviceManager {
    devices: Arc<Mutex<HashMap<DeviceAddress, Device>>>,
    account_devices: Arc<Mutex<HashMap<AccountId, HashSet<DeviceAddress>>>>,
    link_secret: String,
}

impl InMemoryDeviceManager {
    pub fn new(link_secret: String) -> Self {
        InMemoryDeviceManager {
            devices: Arc::new(Mutex::new(HashMap::new())),
            account_devices: Arc::new(Mutex::new(HashMap::new())),
            link_secret,
        }
    }
}

#[async_trait::async_trait]
impl DeviceManager for InMemoryDeviceManager {
    async fn get_device(&self, account_id: AccountId, id: DeviceId) -> Result<Device, ServerError> {
        let key = DeviceAddress::new(account_id, id);
        self.devices
            .lock()
            .await
            .get(&key)
            .ok_or(ServerError::DeviceNotExist)
            .cloned()
    }

    async fn get_devices(&self, account_id: AccountId) -> Result<Vec<DeviceId>, ServerError> {
        let mut devices = vec![];
        if let Some(keys) = self.account_devices.lock().await.get(&account_id) {
            for key in keys {
                let id = self
                    .devices
                    .lock()
                    .await
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

        if self.devices.lock().await.contains_key(&key) {
            return Err(ServerError::DeviceExists);
        }

        self.account_devices
            .lock()
            .await
            .entry(account_id)
            .or_insert_with(HashSet::new);

        self.devices
            .lock()
            .await
            .insert(key.clone(), device.clone());

        if let Some(x) = self.account_devices.lock().await.get_mut(&account_id) {
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

        if let Some(x) = self.account_devices.lock().await.get_mut(&account_id) {
            x.remove(&key);
        }

        self.devices
            .lock()
            .await
            .remove(&key)
            .ok_or(ServerError::AccountNotExist)
            .map(|_| ())
    }
}
