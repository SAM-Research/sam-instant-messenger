use std::collections::HashSet;

use async_trait::async_trait;
use sam_common::{AccountId, DeviceId};

use crate::storage::error::StoreError;

#[async_trait(?Send)]
pub trait ContactStore {
    async fn get_all_devices(&self, account_id: AccountId)
        -> Result<HashSet<DeviceId>, StoreError>;
    async fn add_device(
        &mut self,
        account_id: AccountId,
        device_id: DeviceId,
    ) -> Result<(), StoreError>;
    async fn remove_device(
        &mut self,
        account_id: AccountId,
        device_id: DeviceId,
    ) -> Result<(), StoreError>;

    async fn contains_contact(&self, account_id: AccountId) -> Result<bool, StoreError>;
}
