use crate::storage::error::AccountStoreError;

use async_trait::async_trait;
use sam_common::address::AccountId;
use sam_common::DeviceId;

#[async_trait(?Send)]
pub trait AccountStore {
    async fn set_account_id(&mut self, account_id: AccountId) -> Result<(), AccountStoreError>;
    async fn get_account_id(&self) -> Result<AccountId, AccountStoreError>;
    async fn set_password(&mut self, password: String) -> Result<(), AccountStoreError>;
    async fn get_password(&self) -> Result<String, AccountStoreError>;
    async fn set_username(&mut self, username: String) -> Result<(), AccountStoreError>;
    async fn get_username(&self) -> Result<String, AccountStoreError>;
    async fn set_device_id(&mut self, device_id: DeviceId) -> Result<(), AccountStoreError>;
    async fn get_device_id(&self) -> Result<DeviceId, AccountStoreError>;
}
