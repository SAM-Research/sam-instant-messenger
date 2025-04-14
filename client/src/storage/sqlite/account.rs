use std::str::FromStr as _;

use crate::storage::{error::StoreError, AccountStore};
use async_trait::async_trait;
use log::debug;
use sam_common::address::AccountId;
use sam_common::DeviceId;
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
    async fn set_account_id(&mut self, id: AccountId) -> Result<(), StoreError> {
        let aci = id.to_string();
        sqlx::query!(
            r#"
            DELETE FROM AccountId;
            INSERT INTO AccountId
            VALUES (?)
            "#,
            aci
        )
        .execute(&self.database)
        .await
        .map(|_| ())
        .map_err(|err| StoreError::Database(format!("{err}")))
    }

    async fn get_account_id(&self) -> Result<AccountId, StoreError> {
        match sqlx::query!(
            r#"
            SELECT * FROM AccountId;
            "#,
        )
        .fetch_one(&self.database)
        .await
        .inspect_err(|e| debug!("{e}"))
        {
            Err(SqlxError::RowNotFound) => Err(StoreError::NoAccountId),
            Ok(rec) => AccountId::from_str(&rec.account_id)
                .inspect_err(|e| debug!("{e}"))
                .map_err(|_| StoreError::InvalidAccountId(rec.account_id)),
            Err(err) => Err(StoreError::Database(format!("{err}"))),
        }
    }

    async fn set_password(&mut self, password: String) -> Result<(), StoreError> {
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
        .map_err(|err| StoreError::Database(format!("{err}")))
    }

    async fn get_password(&self) -> Result<String, StoreError> {
        match sqlx::query!(
            r#"
            SELECT * FROM Password;
            "#,
        )
        .fetch_one(&self.database)
        .await
        {
            Err(SqlxError::RowNotFound) => Err(StoreError::NoPassword),
            Ok(rec) => Ok(rec.password),
            Err(err) => Err(StoreError::Database(format!("{err}"))),
        }
    }

    async fn set_username(&mut self, username: String) -> Result<(), StoreError> {
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
        .map_err(|err| StoreError::Database(format!("{err}")))
    }

    async fn get_username(&self) -> Result<String, StoreError> {
        match sqlx::query!(
            r#"
            SELECT * FROM Username;
            "#,
        )
        .fetch_one(&self.database)
        .await
        {
            Err(SqlxError::RowNotFound) => Err(StoreError::NoUsername),
            Ok(rec) => Ok(rec.username),
            Err(err) => Err(StoreError::Database(format!("{err}"))),
        }
    }

    async fn set_device_id(&mut self, device_id: DeviceId) -> Result<(), StoreError> {
        let dev_id = Into::<u32>::into(device_id);
        sqlx::query!(
            r#"
            DELETE FROM DeviceId;
            INSERT INTO DeviceId
            VALUES (?)
            "#,
            dev_id
        )
        .execute(&self.database)
        .await
        .map(|_| ())
        .map_err(|err| StoreError::Database(format!("{err}")))
    }

    async fn get_device_id(&self) -> Result<DeviceId, StoreError> {
        match sqlx::query!(
            r#"
            SELECT * FROM DeviceId;
            "#,
        )
        .fetch_one(&self.database)
        .await
        {
            Err(SqlxError::RowNotFound) => Err(StoreError::NoDeviceId),
            Ok(rec) => Ok((rec.device_id as u32).into()),
            Err(err) => Err(StoreError::Database(format!("{err}"))),
        }
    }
}
