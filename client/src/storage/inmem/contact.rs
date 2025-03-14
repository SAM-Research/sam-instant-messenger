use std::collections::{HashMap, HashSet};

use async_trait::async_trait;
use sam_common::{AccountId, DeviceId};

use crate::{storage::ContactStore, ClientError};

#[derive(Debug, Default)]
pub struct InMemoryContactStore {
    contacts: HashMap<AccountId, HashSet<DeviceId>>,
}

#[async_trait(?Send)]
impl ContactStore for InMemoryContactStore {
    async fn contains_contact(&self, account_id: AccountId) -> Result<bool, ClientError> {
        Ok(self.contacts.contains_key(&account_id))
    }

    async fn get_all_devices(
        &self,
        account_id: AccountId,
    ) -> Result<HashSet<DeviceId>, ClientError> {
        Ok(self.contacts.get(&account_id).cloned().unwrap_or_default())
    }
    async fn add_device(
        &mut self,
        account_id: AccountId,
        device_id: DeviceId,
    ) -> Result<(), ClientError> {
        self.contacts
            .entry(account_id)
            .or_default()
            .insert(device_id);
        Ok(())
    }
    async fn remove_device(
        &mut self,
        account_id: AccountId,
        device_id: DeviceId,
    ) -> Result<(), ClientError> {
        if let Some(devices) = self.contacts.get_mut(&account_id) {
            devices.remove(&device_id);
        }
        Ok(())
    }
}
