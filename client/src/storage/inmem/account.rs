use async_trait::async_trait;
use sam_common::address::AccountId;

use crate::{storage::AccountStore, ClientError};

#[derive(Debug, Default)]
pub struct InMemoryAccountStore {
    username: Option<String>,
    account_id: Option<AccountId>,
    password: Option<String>,
}

#[async_trait(?Send)]
impl AccountStore for InMemoryAccountStore {
    async fn set_account_id(&mut self, account_id: AccountId) -> Result<(), ClientError> {
        self.account_id = Some(account_id);
        Ok(())
    }
    async fn get_account_id(&self) -> Result<AccountId, ClientError> {
        Ok(self.account_id.ok_or(ClientError::NoAccountId)?)
    }
    async fn set_password(&mut self, password: String) -> Result<(), ClientError> {
        self.password = Some(password);
        Ok(())
    }
    async fn get_password(&self) -> Result<String, ClientError> {
        Ok(self.password.clone().ok_or(ClientError::NoPassword)?)
    }
    async fn set_username(&mut self, username: String) -> Result<(), ClientError> {
        self.username = Some(username);
        Ok(())
    }
    async fn get_username(&self) -> Result<String, ClientError> {
        Ok(self.username.clone().ok_or(ClientError::NoUsername)?)
    }
}
