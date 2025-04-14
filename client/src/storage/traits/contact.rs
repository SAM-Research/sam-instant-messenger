use std::collections::HashSet;

use async_trait::async_trait;
use sam_common::{AccountId, DeviceId};

use crate::storage::error::ContactStoreError;

#[async_trait(?Send)]
pub trait ContactStore {
    async fn get_all_devices(
        &self,
        account_id: AccountId,
    ) -> Result<HashSet<DeviceId>, ContactStoreError>;
    async fn add_device(
        &mut self,
        account_id: AccountId,
        device_id: DeviceId,
    ) -> Result<(), ContactStoreError>;
    async fn remove_device(
        &mut self,
        account_id: AccountId,
        device_id: DeviceId,
    ) -> Result<(), ContactStoreError>;

    async fn contains_contact(&self, account_id: AccountId) -> Result<bool, ContactStoreError>;
}
