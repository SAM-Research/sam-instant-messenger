use libsignal_protocol::IdentityKey;
use log::{debug, error};
use sam_common::AccountId;
use sqlx::{types::Uuid, Pool, Postgres};

use crate::{
    managers::{entities::Account, traits::account_manager::AccountManager},
    ServerError,
};

#[derive(Debug, Clone)]
pub struct PostgresAccountManager {
    pool: Pool<Postgres>,
}

#[async_trait::async_trait]
impl AccountManager for PostgresAccountManager {
    async fn get_account(&self, id: AccountId) -> Result<Account, ServerError> {
        let uuid = id.uuid();
        match sqlx::query!(
            r#"
            SELECT * FROM accounts
            WHERE account_id = $1
            "#,
            uuid
        )
        .fetch_one(&self.pool)
        .await
        {
            Ok(row) => {
                let identity = IdentityKey::decode(row.identity_key.as_slice())
                    .inspect_err(|err| error!("Error loading account UUID from database: {err}"))
                    .map_err(|_| ServerError::Database)?;
                Ok(Account::builder()
                    .id(row.account_id.into())
                    .identity(identity)
                    .username(row.username)
                    .build())
            }
            Err(err) => {
                debug!("Could not fetch account from database: {err}");
                Err(ServerError::Database)
            }
        }
    }

    async fn get_account_id_from_username(
        &self,
        username: String,
    ) -> Result<AccountId, ServerError> {
        match sqlx::query!(
            r#"
            SELECT (account_id) FROM accounts
            WHERE username = $1
            "#,
            username
        )
        .fetch_one(&self.pool)
        .await
        {
            Ok(row) => Ok(row.account_id.into()),
            Err(err) => {
                debug!("Could not fetch username from database: {err}");
                Err(ServerError::Database)
            }
        }
    }

    async fn add_account(&mut self, account: &Account) -> Result<(), ServerError> {
        let id: Uuid = account.id().into();
        let username = account.username();
        let identity = account.identity().serialize().to_vec();
        match sqlx::query!(
            r#"
            INSERT INTO accounts (account_id, username, identity_key)
            VALUES ($1, $2, $3)
            "#,
            id,
            username,
            &identity
        )
        .execute(&self.pool)
        .await
        {
            Ok(_) => Ok(()),
            Err(err) => {
                debug!("Could not save account in database: {err}");
                Err(ServerError::Database)
            }
        }
    }

    async fn remove_account(&mut self, account_id: AccountId) -> Result<(), ServerError> {
        let account_id: Uuid = account_id.into();
        match sqlx::query!(
            r#"
            DELETE FROM accounts 
            WHERE account_id = $1
            "#,
            account_id,
        )
        .execute(&self.pool)
        .await
        {
            Ok(_) => Ok(()),
            Err(err) => {
                debug!("Could not remove account from database: {err}");
                Err(ServerError::Database)
            }
        }
    }
}
