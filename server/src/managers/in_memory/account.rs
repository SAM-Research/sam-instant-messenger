use sam_common::address::AccountId;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

use crate::managers::error::AccountManagerError;
use crate::managers::{entities::account::Account, traits::account_manager::AccountManager};

#[derive(Clone)]
pub struct InMemoryAccountManager {
    accounts: Arc<Mutex<HashMap<AccountId, Account>>>,
}

impl Default for InMemoryAccountManager {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryAccountManager {
    pub fn new() -> Self {
        InMemoryAccountManager {
            accounts: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait::async_trait]
impl AccountManager for InMemoryAccountManager {
    async fn get_account(&self, id: AccountId) -> Result<Account, AccountManagerError> {
        Ok(self
            .accounts
            .lock()
            .await
            .get(&id)
            .cloned()
            .ok_or(AccountManagerError::AccountDoesNotExist)?)
    }

    async fn get_account_id_from_username(
        &self,
        username: String,
    ) -> Result<AccountId, AccountManagerError> {
        let account = self
            .accounts
            .lock()
            .await
            .values()
            .find(|acc| acc.username() == username)
            .cloned()
            .ok_or(AccountManagerError::AccountDoesNotExist)?;

        Ok(account.id())
    }

    async fn add_account(&mut self, account: &Account) -> Result<(), AccountManagerError> {
        if self.accounts.lock().await.contains_key(&account.id()) {
            return Err(AccountManagerError::AccountAlreadyExists)?;
        } else if self
            .accounts
            .lock()
            .await
            .values()
            .any(|acc| acc.username() == account.username())
        {
            return Err(AccountManagerError::UsernameAlreadyExists)?;
        }
        self.accounts
            .lock()
            .await
            .insert(account.id(), account.clone());
        Ok(())
    }

    async fn remove_account(&mut self, account_id: AccountId) -> Result<(), AccountManagerError> {
        let _ = self
            .accounts
            .lock()
            .await
            .remove(&account_id)
            .ok_or(AccountManagerError::AccountDoesNotExist)?;
        Ok(())
    }
}
