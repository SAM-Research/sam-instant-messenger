use async_trait::async_trait;
use sam_common::address::AccountId;
use std::collections::{hash_map::Entry, HashMap};

use crate::storage::{account::Account, error::AccountStoreError, traits::AccountStore};

#[derive(Debug)]
pub struct InMemoryAccountStore {
    accounts: HashMap<AccountId, Account>,
}

#[async_trait(?Send)]
impl AccountStore for InMemoryAccountStore {
    async fn add_account(&mut self, account: Account) -> Result<(), AccountStoreError> {
        if self
            .accounts
            .insert(account.aci().to_owned(), account.to_owned())
            .is_some()
        {
            return Err(AccountStoreError::AccountIdTaken(account.aci().to_string()));
        }
        Ok(())
    }

    async fn get_account(&self, _service_id: &AccountId) -> Result<Account, AccountStoreError> {
        self.accounts
            .get(_service_id)
            .ok_or(AccountStoreError::AccountNotFound(_service_id.to_owned()))
            .map(|acc| acc.to_owned())
    }

    async fn update_account_identifier(
        &mut self,
        service_id: &AccountId,
        _new_aci: &AccountId,
    ) -> Result<(), AccountStoreError> {
        let account = self
            .accounts
            .remove(service_id)
            .ok_or(AccountStoreError::AccountNotFound(service_id.to_owned()))?;
        match self.accounts.entry(service_id.to_owned()) {
            Entry::Occupied(_) => Err(AccountStoreError::AccountIdTaken(service_id.to_string())),
            Entry::Vacant(vacant_entry) => {
                vacant_entry.insert(account);
                Ok(())
            }
        }
    }

    async fn delete_account(&mut self, service_id: &AccountId) -> Result<(), AccountStoreError> {
        self.accounts
            .remove(service_id)
            .ok_or(AccountStoreError::AccountNotFound(service_id.to_owned()))
            .map(|_| ())
    }

    async fn add_used_device_link_token(
        &mut self,
        _device_link_token: String,
    ) -> Result<(), AccountStoreError> {
        todo!()
    }
}
