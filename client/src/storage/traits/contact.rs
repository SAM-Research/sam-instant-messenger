use std::collections::HashSet;

use async_trait::async_trait;
use sam_common::{AccountId, DeviceId};

use crate::ClientError;

#[async_trait(?Send)]
pub trait ContactStore {
    async fn get_all_devices(
        &self,
        account_id: AccountId,
    ) -> Result<HashSet<DeviceId>, ClientError>;
    async fn add_device(
        &mut self,
        account_id: AccountId,
        device_id: DeviceId,
    ) -> Result<(), ClientError>;
    async fn remove_device(
        &mut self,
        account_id: AccountId,
        device_id: DeviceId,
    ) -> Result<(), ClientError>;
}
