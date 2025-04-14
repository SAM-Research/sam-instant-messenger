use crate::storage::error::AccountStoreError;
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
    async fn set_account_id(&mut self, account_id: AccountId) -> Result<(), AccountStoreError> {
        self.account_id = Some(account_id);
        Ok(())
    }
    async fn get_account_id(&self) -> Result<AccountId, AccountStoreError> {
        Ok(self.account_id.ok_or(AccountStoreError::NoAccountId)?)
    }
    async fn set_password(&mut self, password: String) -> Result<(), AccountStoreError> {
        self.password = Some(password);
        Ok(())
    }
    async fn get_password(&self) -> Result<String, AccountStoreError> {
        Ok(self.password.clone().ok_or(AccountStoreError::NoPassword)?)
    }
    async fn set_username(&mut self, username: String) -> Result<(), AccountStoreError> {
        self.username = Some(username);
        Ok(())
    }
    async fn get_username(&self) -> Result<String, AccountStoreError> {
        Ok(self.username.clone().ok_or(AccountStoreError::NoUsername)?)
    }

    async fn set_device_id(&mut self, device_id: DeviceId) -> Result<(), AccountStoreError> {
        self.device_id = Some(device_id);
        Ok(())
    }

    async fn get_device_id(&self) -> Result<DeviceId, AccountStoreError> {
        Ok(self.device_id.ok_or(AccountStoreError::NoDeviceId)?)
    }
}
