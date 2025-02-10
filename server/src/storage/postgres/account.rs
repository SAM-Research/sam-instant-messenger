use crate::storage::{error::AccountStoreError, Account, AccountStore};
use async_trait::async_trait;
use libsignal_protocol::{Aci, IdentityKey, PublicKey};
use sqlx::{Pool, Postgres};

#[derive(Debug)]
pub struct PostgresAccountStore {
    database: Pool<Postgres>,
}

#[async_trait(?Send)]
impl AccountStore for PostgresAccountStore {
    async fn add_account(&mut self, account: &Account) -> Result<(), AccountStoreError> {
        sqlx::query!(
            r#"
            INSERT INTO accounts (aci, aci_identity_key)
            VALUES ($1, $2)
            "#,
            account.aci().service_id_string(),
            &*account.identity_key().serialize(),
        )
        .execute(&self.database)
        .await
        .map(|_| ())
        .map_err(|err| AccountStoreError::Database(err.into()))
    }

    async fn get_account(&self, service_id: &Aci) -> Result<Account, AccountStoreError> {
        sqlx::query!(
            r#"
            SELECT aci, 
                   aci_identity_key
            FROM accounts
            WHERE aci = $1 
            "#,
            service_id.service_id_string(),
        )
        .fetch_one(&self.database)
        .await
        .map(|row| {
            Account::builder()
                .aci(Aci::parse_from_service_id_string(&row.aci).unwrap())
                .identity_key(IdentityKey::new(
                    PublicKey::deserialize(row.aci_identity_key.as_slice()).unwrap(),
                ))
                .build()
        })
        .map_err(|err| AccountStoreError::Database(err.into()))
    }

    async fn update_account_identifier(
        &mut self,
        service_id: &Aci,
        new_aci: Aci,
    ) -> Result<(), AccountStoreError> {
        sqlx::query!(
            r#"
            UPDATE accounts
            SET aci = $2
            WHERE aci = $1 
            "#,
            service_id.service_id_string(),
            new_aci.service_id_string()
        )
        .execute(&self.database)
        .await
        .map(|_| ())
        .map_err(|err| AccountStoreError::Database(err.into()))
    }

    async fn delete_account(&mut self, service_id: &Aci) -> Result<(), AccountStoreError> {
        sqlx::query!(
            r#"
            DELETE 
            FROM accounts
            WHERE aci = $1 
            "#,
            service_id.service_id_string()
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
