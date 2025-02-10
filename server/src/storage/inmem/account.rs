use std::collections::{hash_map::Entry, HashMap};

use async_trait::async_trait;
use libsignal_protocol::Aci;

use crate::storage::{error::AccountStoreError, Account, AccountStore};

#[derive(Debug)]
pub struct InMemoryAccountStore {
    accounts: HashMap<Aci, Account>,
}

#[async_trait(?Send)]
impl AccountStore for InMemoryAccountStore {
    async fn add_account(&mut self, account: &Account) -> Result<(), AccountStoreError> {
        if self
            .accounts
            .insert(account.aci, account.to_owned())
            .is_some()
        {
            return Err(AccountStoreError::AccountIdTaken(
                account.aci().service_id_string(),
            ));
        }
        Ok(())
    }

    async fn get_account(&self, _service_id: &Aci) -> Result<Account, AccountStoreError> {
        self.accounts
            .get(_service_id)
            .ok_or(AccountStoreError::AccountNotFound(
                _service_id.service_id_string(),
            ))
            .map(|acc| acc.to_owned())
    }

    async fn update_account_identifier(
        &mut self,
        service_id: &Aci,
        _new_aci: Aci,
    ) -> Result<(), AccountStoreError> {
        let account =
            self.accounts
                .remove(service_id)
                .ok_or(AccountStoreError::AccountNotFound(
                    service_id.service_id_string(),
                ))?;
        match self.accounts.entry(*service_id) {
            Entry::Occupied(_) => Err(AccountStoreError::AccountIdTaken(
                service_id.service_id_string(),
            )),
            Entry::Vacant(vacant_entry) => {
                vacant_entry.insert(account);
                Ok(())
            }
        }
    }

    async fn delete_account(&mut self, service_id: &Aci) -> Result<(), AccountStoreError> {
        self.accounts
            .remove(service_id)
            .ok_or(AccountStoreError::AccountNotFound(
                service_id.service_id_string(),
            ))
            .map(|_| ())
    }

    async fn add_used_device_link_token(
        &mut self,
        _device_link_token: String,
    ) -> Result<(), AccountStoreError> {
        todo!()
    }
}
