use async_trait::async_trait;
use sam_common::address::AccountId;

use crate::ClientError;

#[async_trait(?Send)]
pub trait AccountStore {
    async fn set_account_id(&mut self, account_id: AccountId) -> Result<(), ClientError>;
    async fn get_account_id(&self) -> Result<AccountId, ClientError>;
    async fn set_password(&mut self, password: String) -> Result<(), ClientError>;
    async fn get_password(&self) -> Result<String, ClientError>;
    async fn set_username(&mut self, username: String) -> Result<(), ClientError>;
    async fn get_username(&self) -> Result<String, ClientError>;
}
