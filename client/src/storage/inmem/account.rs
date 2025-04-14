use crate::storage::error::StoreError;
use crate::storage::AccountStore;
use async_trait::async_trait;
use sam_common::address::AccountId;
use sam_common::DeviceId;

#[derive(Debug, Default)]
pub struct InMemoryAccountStore {
    username: Option<String>,
    account_id: Option<AccountId>,
    device_id: Option<DeviceId>,
    password: Option<String>,
}

#[async_trait(?Send)]
impl AccountStore for InMemoryAccountStore {
    async fn set_account_id(&mut self, account_id: AccountId) -> Result<(), StoreError> {
        self.account_id = Some(account_id);
        Ok(())
    }
    async fn get_account_id(&self) -> Result<AccountId, StoreError> {
        Ok(self.account_id.ok_or(StoreError::NoAccountId)?)
    }
    async fn set_password(&mut self, password: String) -> Result<(), StoreError> {
        self.password = Some(password);
        Ok(())
    }
    async fn get_password(&self) -> Result<String, StoreError> {
        Ok(self.password.clone().ok_or(StoreError::NoPassword)?)
    }
    async fn set_username(&mut self, username: String) -> Result<(), StoreError> {
        self.username = Some(username);
        Ok(())
    }
    async fn get_username(&self) -> Result<String, StoreError> {
        Ok(self.username.clone().ok_or(StoreError::NoUsername)?)
    }

    async fn set_device_id(&mut self, device_id: DeviceId) -> Result<(), StoreError> {
        self.device_id = Some(device_id);
        Ok(())
    }

    async fn get_device_id(&self) -> Result<DeviceId, StoreError> {
        Ok(self.device_id.ok_or(StoreError::NoDeviceId)?)
    }
}
