use crate::{storage::AccountStore, ClientError};
use async_trait::async_trait;
use libsignal_protocol::Aci;
use sqlx::{Pool, Sqlite};

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
    async fn set_aci(&self, aci: Aci) -> Result<(), ClientError> {
        let aci = aci.service_id_string();
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

    async fn get_aci(&self) -> Result<Aci, ClientError> {
        sqlx::query!(
            r#"
            SELECT * FROM Aci;
            "#,
        )
        .fetch_one(&self.database)
        .await
        .map(|rec| {
            Aci::parse_from_service_id_string(&rec.aci)
                .ok_or(ClientError::InvalidServiceId(rec.aci))
        })?
        .map_err(ClientError::from)
    }

    async fn set_password(&self, password: String) -> Result<(), ClientError> {
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
        sqlx::query!(
            r#"
            SELECT * FROM Password;
            "#,
        )
        .fetch_one(&self.database)
        .await
        .map(|rec| rec.password)
        .map_err(ClientError::from)
    }

    async fn set_username(&self, username: String) -> Result<(), ClientError> {
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
        sqlx::query!(
            r#"
            SELECT * FROM Username;
            "#,
        )
        .fetch_one(&self.database)
        .await
        .map(|rec| rec.username)
        .map_err(ClientError::from)
    }
}
