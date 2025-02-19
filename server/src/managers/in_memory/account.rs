use sam_common::address::AccountId;
use std::{collections::HashMap, sync::Arc};

use tokio::sync::Mutex;

use crate::{
    managers::{entities::account::Account, traits::account_manager::AccountManager},
    ServerError,
};

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
    async fn get_account(&self, id: AccountId) -> Result<Account, ServerError> {
        self.accounts
            .lock()
            .await
            .get(&id)
            .ok_or(ServerError::AccountNotExist)
            .cloned()
    }

    async fn add_account(&mut self, account: &Account) -> Result<(), ServerError> {
        if self.accounts.lock().await.contains_key(&account.id()) {
            return Err(ServerError::AccountExists);
        }
        self.accounts
            .lock()
            .await
            .insert(account.id(), account.clone());
        Ok(())
    }

    async fn remove_account(&mut self, account_id: AccountId) -> Result<(), ServerError> {
        self.accounts
            .lock()
            .await
            .remove(&account_id)
            .ok_or(ServerError::AccountNotExist)
            .map(|_| ())
    }
}
