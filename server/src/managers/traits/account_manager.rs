use sam_common::address::AccountId;

use crate::{managers::entities::account::Account, ServerError};

#[async_trait::async_trait]
pub trait AccountManager: Send + Sync + Clone {
    async fn get_account(&self, id: AccountId) -> Result<Account, ServerError>;
    async fn add_account(&mut self, account: &Account) -> Result<(), ServerError>;
    async fn remove_account(&mut self, account_id: AccountId) -> Result<(), ServerError>;
}
