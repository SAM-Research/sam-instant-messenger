use std::collections::HashMap;

use uuid::Uuid;

use crate::{
    managers::{entities::account::Account, traits::account_manager::AccountManager},
    ServerError,
};

pub struct InMemoryAccountManager {
    accounts: HashMap<Uuid, Account>,
}

impl InMemoryAccountManager {
    pub fn new() -> Self {
        InMemoryAccountManager {
            accounts: HashMap::new(),
        }
    }
}

#[async_trait::async_trait]
impl AccountManager for InMemoryAccountManager {
    async fn get_account(&self, id: &Uuid) -> Result<Account, ServerError> {
        self.accounts
            .get(id)
            .ok_or(ServerError::AccountNotExist)
            .cloned()
    }

    async fn add_account(&mut self, account: &Account) -> Result<(), ServerError> {
        if self.accounts.contains_key(account.id()) {
            return Err(ServerError::AccountExists);
        }
        self.accounts.insert(*account.id(), account.clone());
        Ok(())
    }

    async fn remove_account(&mut self, account_id: Uuid) -> Result<(), ServerError> {
        self.accounts
            .remove(&account_id)
            .ok_or(ServerError::AccountNotExist)
            .map(|_| ())
    }
}
