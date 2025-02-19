use async_trait::async_trait;
use base64::{prelude::BASE64_STANDARD, Engine};
use libsignal_protocol::{PreKeyId, PreKeyRecord, PreKeyStore, SignalProtocolError};
use sqlx::{Pool, Sqlite};

use crate::{
    storage::{error::DatabaseError, ProvidesKeyId},
    ClientError,
};

#[derive(Debug)]
pub struct SqlitePreKeyStore {
    database: Pool<Sqlite>,
}

impl SqlitePreKeyStore {
    pub fn new(database: Pool<Sqlite>) -> Self {
        Self { database }
    }
}

#[async_trait(?Send)]
impl ProvidesKeyId<PreKeyId> for SqlitePreKeyStore {
    async fn next_key_id(&self) -> Result<PreKeyId, ClientError> {
        sqlx::query!(
            r#"
            WITH max_pre_key_id_table AS (
                SELECT
                    1 AS _id,
                    MAX(id) AS max_pre_key_id
                FROM
                    DeviceSignedPreKeyStore
                )
                SELECT
                    CASE WHEN pk.max_pre_key_id IS NOT NULL
                    THEN pk.max_pre_key_id
                    ELSE
                    0
                    END AS pkid
                FROM
                    max_pre_key_id_table pk
                "#
        )
        .fetch_one(&self.database)
        .await
        .map(|row| PreKeyId::from(row.pkid as u32))
        .map_err(ClientError::from)
    }
}

#[async_trait(?Send)]
impl PreKeyStore for SqlitePreKeyStore {
    async fn get_pre_key(&self, prekey_id: PreKeyId) -> Result<PreKeyRecord, SignalProtocolError> {
        let id: u32 = prekey_id.into();

        match sqlx::query!(
            r#"
            SELECT
                pre_key_record
            FROM
                DevicePreKeyStore
            WHERE
                pre_key_id = ?
            "#,
            id
        )
        .fetch_one(&self.database)
        .await
        {
            Ok(row) => PreKeyRecord::deserialize(
                BASE64_STANDARD
                    .decode(row.pre_key_record)
                    .map_err(|err| {
                        SignalProtocolError::ApplicationCallbackError(
                            "decode pre key from base64",
                            Box::new(err),
                        )
                    })?
                    .as_slice(),
            )
            .map_err(|err| {
                SignalProtocolError::ApplicationCallbackError(
                    "deserialize pre key from bytes",
                    Box::new(err),
                )
            }),

            Err(err) => Err(SignalProtocolError::ApplicationCallbackError(
                "save pre key",
                Box::new(DatabaseError::from(err)),
            )),
        }
    }

    async fn save_pre_key(
        &mut self,
        prekey_id: PreKeyId,
        record: &PreKeyRecord,
    ) -> Result<(), SignalProtocolError> {
        let id: u32 = prekey_id.into();
        let rec = BASE64_STANDARD.encode(record.serialize()?);

        sqlx::query!(
            r#"
            INSERT INTO DevicePreKeyStore (pre_key_id, pre_key_record)
            VALUES (?, ?)
            ON CONFLICT(pre_key_id) DO UPDATE SET pre_key_record = ?
            "#,
            id,
            rec,
            rec
        )
        .execute(&self.database)
        .await
        .map(|_| ())
        .map_err(|err| {
            SignalProtocolError::ApplicationCallbackError(
                "save pre key",
                Box::new(DatabaseError::from(err)),
            )
        })
    }

    async fn remove_pre_key(&mut self, prekey_id: PreKeyId) -> Result<(), SignalProtocolError> {
        let id: u32 = prekey_id.into();

        sqlx::query!(
            r#"
            DELETE FROM
                DevicePreKeyStore
            WHERE
                pre_key_id = ?
            "#,
            id
        )
        .execute(&self.database)
        .await
        .map(|_| ())
        .map_err(|err| {
            SignalProtocolError::ApplicationCallbackError(
                "remove pre key",
                Box::new(DatabaseError::from(err)),
            )
        })
    }
}
