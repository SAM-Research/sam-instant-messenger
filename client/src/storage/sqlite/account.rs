use std::str::FromStr as _;

use crate::{storage::AccountStore, ClientError};
use async_trait::async_trait;
use sam_common::address::AccountId;
use sqlx::{Error as SqlxError, Pool, Sqlite};

#[derive(Debug)]
pub struct SqliteAccountStore {
    database: Pool<Sqlite>,
}

impl SqliteAccountStore {
    pub fn new(database: Pool<Sqlite>) -> Self {
        Self { database }
    }
}

#[async_trait(?Send)]
impl AccountStore for SqliteAccountStore {
    async fn set_account_id(&mut self, aci: AccountId) -> Result<(), ClientError> {
        let aci = aci.to_string();
        sqlx::query!(
            r#"
            DELETE FROM Aci;
            INSERT INTO Aci
            VALUES (?)
            "#,
            aci
        )
        .execute(&self.database)
        .await
        .map(|_| ())
        .map_err(ClientError::from)
    }

    async fn get_account_id(&self) -> Result<AccountId, ClientError> {
        match sqlx::query!(
            r#"
            SELECT * FROM Aci;
            "#,
        )
        .fetch_one(&self.database)
        .await
        {
            Err(SqlxError::RowNotFound) => Err(ClientError::NoAccountId),
            Ok(rec) => {
                AccountId::from_str(&rec.aci).map_err(|_| ClientError::InvalidServiceId(rec.aci))
            }
            Err(err) => Err(ClientError::from(err)),
        }
    }

    async fn set_password(&mut self, password: String) -> Result<(), ClientError> {
        sqlx::query!(
            r#"
            DELETE FROM Password;
            INSERT INTO Password
            VALUES (?)
            "#,
            password
        )
        .execute(&self.database)
        .await
        .map(|_| ())
        .map_err(ClientError::from)
    }

    async fn get_password(&self) -> Result<String, ClientError> {
        match sqlx::query!(
            r#"
            SELECT * FROM Password;
            "#,
        )
        .fetch_one(&self.database)
        .await
        {
            Err(SqlxError::RowNotFound) => Err(ClientError::NoPassword),
            Ok(rec) => Ok(rec.password),
            Err(err) => Err(ClientError::from(err)),
        }
    }

    async fn set_username(&mut self, username: String) -> Result<(), ClientError> {
        sqlx::query!(
            r#"
            DELETE FROM Username;
            INSERT INTO Username
            VALUES (?)
            "#,
            username
        )
        .execute(&self.database)
        .await
        .map(|_| ())
        .map_err(ClientError::from)
    }

    async fn get_username(&self) -> Result<String, ClientError> {
        match sqlx::query!(
            r#"
            SELECT * FROM Username;
            "#,
        )
        .fetch_one(&self.database)
        .await
        {
            Err(SqlxError::RowNotFound) => Err(ClientError::NoUsername),
            Ok(rec) => Ok(rec.username),
            Err(err) => Err(ClientError::from(err)),
        }
    }
}
