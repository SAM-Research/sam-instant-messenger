use std::str::FromStr;

use crate::storage::{
    account::Account, error::AccountStoreError, traits::AccountStore, IdentityKey, PublicKey as _,
};
use async_trait::async_trait;
use sam_common::address::AccountId;
use sqlx::{Pool, Postgres};

#[derive(Debug)]
pub struct PostgresAccountStore {
    database: Pool<Postgres>,
}

#[async_trait(?Send)]
impl AccountStore for PostgresAccountStore {
    async fn add_account(&mut self, account: Account) -> Result<(), AccountStoreError> {
        sqlx::query!(
            r#"
            INSERT INTO accounts (aci, aci_identity_key)
            VALUES ($1, $2)
            "#,
            account.aci().to_string(),
            &*account.identity_key().serialize(),
        )
        .execute(&self.database)
        .await
        .map(|_| ())
        .map_err(|err| AccountStoreError::Database(err.into()))
    }

    async fn get_account(&self, service_id: &AccountId) -> Result<Account, AccountStoreError> {
        sqlx::query!(
            r#"
            SELECT aci, 
                   aci_identity_key
            FROM accounts
            WHERE aci = $1 
            "#,
            service_id.to_string(),
        )
        .fetch_one(&self.database)
        .await
        .map(|row| {
            Account::builder()
                .aci(AccountId::from_str(&row.aci).unwrap())
                .identity_key(IdentityKey::deserialize(row.aci_identity_key.as_slice()))
                .build()
        })
        .map_err(|err| AccountStoreError::Database(err.into()))
    }

    async fn update_account_identifier(
        &mut self,
        service_id: &AccountId,
        new_aci: &AccountId,
    ) -> Result<(), AccountStoreError> {
        sqlx::query!(
            r#"
            UPDATE accounts
            SET aci = $2
            WHERE aci = $1 
            "#,
            service_id.to_string(),
            new_aci.to_string()
        )
        .execute(&self.database)
        .await
        .map(|_| ())
        .map_err(|err| AccountStoreError::Database(err.into()))
    }

    async fn delete_account(&mut self, service_id: &AccountId) -> Result<(), AccountStoreError> {
        sqlx::query!(
            r#"
            DELETE 
            FROM accounts
            WHERE aci = $1 
            "#,
            service_id.to_string()
        )
        .execute(&self.database)
        .await
        .map(|_| ())
        .map_err(|err| AccountStoreError::Database(err.into()))
    }

    async fn add_used_device_link_token(
        &mut self,
        _device_link_token: String,
    ) -> Result<(), AccountStoreError> {
        todo!()
    }
}
