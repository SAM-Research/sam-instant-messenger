use sam_common::AccountId;

use crate::{
    managers::{entities::Account, traits::account_manager::AccountManager},
    ServerError,
};

#[derive(Debug, Clone)]
pub struct PostgresAccountManager {}

#[async_trait::async_trait]
impl AccountManager for PostgresAccountManager {
    async fn get_account(&self, _id: AccountId) -> Result<Account, ServerError> {
        todo!()
    }
    async fn get_account_id_from_username(
        &self,
        _username: String,
    ) -> Result<AccountId, ServerError> {
        todo!()
    }
    async fn add_account(&mut self, _account: &Account) -> Result<(), ServerError> {
        todo!()
    }
    async fn remove_account(&mut self, _account_id: AccountId) -> Result<(), ServerError> {
        todo!()
    }
}
