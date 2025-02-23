use async_trait::async_trait;
use base64::{prelude::BASE64_STANDARD, Engine as _};
use libsignal_protocol::{
    GenericSignedPreKey as _, KyberPreKeyId, KyberPreKeyRecord, KyberPreKeyStore,
    SignalProtocolError,
};
use sqlx::{Pool, Sqlite};

use crate::{storage::ProvidesKeyId, ClientError};

#[derive(Debug)]
pub struct SqliteKyberPreKeyStore {
    database: Pool<Sqlite>,
}

impl SqliteKyberPreKeyStore {
    pub fn new(database: Pool<Sqlite>) -> Self {
        Self { database }
    }
}

#[async_trait(?Send)]
impl ProvidesKeyId<KyberPreKeyId> for SqliteKyberPreKeyStore {
    async fn next_key_id(&self) -> Result<KyberPreKeyId, ClientError> {
        sqlx::query!(
            r#"
            WITH max_signed_pre_key_id_table AS (
                SELECT
                    1 AS _id,
                    MAX(signed_pre_key_id) AS max_signed_pre_key_id
                FROM
                    DeviceSignedPreKeyStore
                )
                SELECT
                    CASE WHEN spk.max_signed_pre_key_id IS NOT NULL
                    THEN spk.max_signed_pre_key_id
                    ELSE
                    0
                    END AS spkid
                FROM
                    max_signed_pre_key_id_table spk
                "#
        )
        .fetch_one(&self.database)
        .await
        .map(|row| KyberPreKeyId::from(row.spkid as u32))
        .map_err(ClientError::from)
    }
}

#[async_trait(?Send)]
impl KyberPreKeyStore for SqliteKyberPreKeyStore {
    async fn get_kyber_pre_key(
        &self,
        kyber_prekey_id: KyberPreKeyId,
    ) -> Result<KyberPreKeyRecord, SignalProtocolError> {
        let id: u32 = kyber_prekey_id.into();

        match sqlx::query!(
            r#"
            SELECT
                kyber_pre_key_record
            FROM
                DeviceKyberPreKeyStore
            WHERE
                kyber_pre_key_id = ?
            "#,
            id
        )
        .fetch_one(&self.database)
        .await
        {
            Ok(row) => KyberPreKeyRecord::deserialize(
                BASE64_STANDARD
                    .decode(row.kyber_pre_key_record)
                    .map_err(|err| {
                        SignalProtocolError::ApplicationCallbackError(
                            "get kyber pre key",
                            Box::new(err),
                        )
                    })?
                    .as_slice(),
            ),
            Err(_) => Err(SignalProtocolError::InvalidKyberPreKeyId),
        }
    }

    async fn save_kyber_pre_key(
        &mut self,
        kyber_prekey_id: KyberPreKeyId,
        record: &KyberPreKeyRecord,
    ) -> Result<(), SignalProtocolError> {
        let id: u32 = kyber_prekey_id.into();
        let rec = BASE64_STANDARD.encode(record.serialize()?);

        sqlx::query!(
            r#"
            INSERT INTO DeviceKyberPreKeyStore (kyber_pre_key_id, kyber_pre_key_record)
            VALUES (?, ?)
            ON CONFLICT(kyber_pre_key_id) DO UPDATE SET kyber_pre_key_record = ?
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
                "save kyber pre key",
                Box::new(ClientError::from(err)),
            )
        })
    }

    async fn mark_kyber_pre_key_used(
        &mut self,
        _kyber_prekey_id: KyberPreKeyId,
    ) -> Result<(), SignalProtocolError> {
        // TODO: Why does Signal have this??
        Ok(())
    }
}
