use async_trait::async_trait;
use base64::{prelude::BASE64_STANDARD, Engine as _};
use libsignal_protocol::{
    GenericSignedPreKey as _, SignalProtocolError, SignedPreKeyId, SignedPreKeyRecord,
    SignedPreKeyStore,
};
use sqlx::{Pool, Sqlite};

use crate::{storage::ProvidesKeyId, ClientError};

#[derive(Debug)]
pub struct SqliteSignedPreKeyStore {
    database: Pool<Sqlite>,
}

impl SqliteSignedPreKeyStore {
    pub fn new(database: Pool<Sqlite>) -> Self {
        Self { database }
    }
}

#[async_trait(?Send)]
impl ProvidesKeyId<SignedPreKeyId> for SqliteSignedPreKeyStore {
    async fn next_key_id(&self) -> Result<SignedPreKeyId, ClientError> {
        sqlx::query!(
            r#"
            WITH max_signed_pre_key_id_table AS (
                SELECT
                    1 AS _id,
                    MAX(id) AS max_signed_pre_key_id
                FROM
                    DeviceSignedPreKeyStore
                )
                SELECT
                    CASE WHEN pk.max_signed_pre_key_id IS NOT NULL
                    THEN pk.max_signed_pre_key_id
                    ELSE
                    0
                    END AS pkid
                FROM
                    max_signed_pre_key_id_table pk
                "#
        )
        .fetch_one(&self.database)
        .await
        .map(|row| SignedPreKeyId::from(row.pkid as u32))
        .map_err(ClientError::from)
    }
}

#[async_trait(?Send)]
impl SignedPreKeyStore for SqliteSignedPreKeyStore {
    async fn get_signed_pre_key(
        &self,
        id: SignedPreKeyId,
    ) -> Result<SignedPreKeyRecord, SignalProtocolError> {
        let sid: u32 = id.into();

        match sqlx::query!(
            r#"
            SELECT
                signed_pre_key_record
            FROM
                DeviceSignedPreKeyStore
            WHERE
                signed_pre_key_id = ?
            "#,
            sid
        )
        .fetch_one(&self.database)
        .await
        {
            Ok(row) => SignedPreKeyRecord::deserialize(
                BASE64_STANDARD
                    .decode(row.signed_pre_key_record)
                    .map_err(|err| {
                        SignalProtocolError::ApplicationCallbackError(
                            "decode signed pre key",
                            Box::new(err),
                        )
                    })?
                    .as_slice(),
            )
            .map_err(|err| {
                SignalProtocolError::ApplicationCallbackError(
                    "deserialize signed pre key",
                    Box::new(err),
                )
            }),
            Err(err) => Err(SignalProtocolError::ApplicationCallbackError(
                "save signed pre key",
                Box::new(ClientError::from(err)),
            )),
        }
    }

    async fn save_signed_pre_key(
        &mut self,
        id: SignedPreKeyId,
        record: &SignedPreKeyRecord,
    ) -> Result<(), SignalProtocolError> {
        let id: u32 = id.into();
        let rec = BASE64_STANDARD.encode(record.serialize()?);

        sqlx::query!(
            r#"
            INSERT INTO DeviceSignedPreKeyStore (signed_pre_key_id, signed_pre_key_record)
            VALUES (?, ?)
            ON CONFLICT(signed_pre_key_id) DO UPDATE SET signed_pre_key_record = ?
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
                "save signed pre key",
                Box::new(ClientError::from(err)),
            )
        })
    }
}
