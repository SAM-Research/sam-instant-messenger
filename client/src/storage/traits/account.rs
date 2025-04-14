use crate::storage::error::StoreError;

use async_trait::async_trait;
use sam_common::address::AccountId;
use sam_common::DeviceId;

#[async_trait(?Send)]
pub trait AccountStore {
    async fn set_account_id(&mut self, account_id: AccountId) -> Result<(), StoreError>;
    async fn get_account_id(&self) -> Result<AccountId, StoreError>;
    async fn set_password(&mut self, password: String) -> Result<(), StoreError>;
    async fn get_password(&self) -> Result<String, StoreError>;
    async fn set_username(&mut self, username: String) -> Result<(), StoreError>;
    async fn get_username(&self) -> Result<String, StoreError>;
    async fn set_device_id(&mut self, device_id: DeviceId) -> Result<(), StoreError>;
    async fn get_device_id(&self) -> Result<DeviceId, StoreError>;
}
