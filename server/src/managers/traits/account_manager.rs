use async_trait::async_trait;
use sam_common::address::AccountId;

use crate::managers::{entities::account::Account, error::AccountManagerError};

#[async_trait]
pub trait AccountManager: Send + Sync + Clone {
    async fn get_account(&self, id: AccountId) -> Result<Account, AccountManagerError>;
    async fn get_account_id_from_username(
        &self,
        username: String,
    ) -> Result<AccountId, AccountManagerError>;
    async fn add_account(&mut self, account: &Account) -> Result<(), AccountManagerError>;
    async fn remove_account(&mut self, account_id: AccountId) -> Result<(), AccountManagerError>;
}
