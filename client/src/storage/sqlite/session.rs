use async_trait::async_trait;
use base64::{prelude::BASE64_STANDARD, Engine as _};
use libsignal_protocol::{ProtocolAddress, SessionRecord, SessionStore, SignalProtocolError};
use sqlx::{Pool, Sqlite};

use crate::storage::error::DatabaseError;

#[derive(Debug)]
pub struct SqliteSessionStore {
    database: Pool<Sqlite>,
}

impl SqliteSessionStore {
    pub fn new(database: Pool<Sqlite>) -> Self {
        Self { database }
    }
}

#[async_trait(?Send)]
impl SessionStore for SqliteSessionStore {
    async fn load_session(
        &self,
        address: &ProtocolAddress,
    ) -> Result<Option<SessionRecord>, SignalProtocolError> {
        let addr = format!("{}", address);

        match sqlx::query!(
            r#"
            SELECT
                session_record
            FROM
                DeviceSessionStore
            WHERE
                address = ?
            "#,
            addr
        )
        .fetch_one(&self.database)
        .await
        {
            Ok(row) => SessionRecord::deserialize(
                BASE64_STANDARD
                    .decode(row.session_record)
                    .map_err(|err| {
                        SignalProtocolError::ApplicationCallbackError(
                            "load session from database",
                            Box::new(err),
                        )
                    })?
                    .as_slice(),
            )
            .map(Some),
            Err(_) => Ok(None),
        }
    }
    async fn store_session(
        &mut self,
        address: &ProtocolAddress,
        record: &SessionRecord,
    ) -> Result<(), SignalProtocolError> {
        let addr = format!("{}", address);
        let rec = BASE64_STANDARD.encode(record.serialize()?);

        sqlx::query!(
            r#"
            INSERT INTO DeviceSessionStore (address, session_record)
            VALUES (?, ?)
            ON CONFLICT(address) DO UPDATE SET session_record = ?
            "#,
            addr,
            rec,
            rec
        )
        .execute(&self.database)
        .await
        .map(|_| ())
        .map_err(|err| {
            SignalProtocolError::ApplicationCallbackError(
                "store session",
                Box::new(DatabaseError::from(err)),
            )
        })
    }
}
