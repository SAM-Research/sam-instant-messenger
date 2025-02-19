use async_trait::async_trait;
use base64::{prelude::BASE64_STANDARD, Engine as _};
use libsignal_protocol::{ProtocolAddress, SenderKeyRecord, SenderKeyStore, SignalProtocolError};
use sqlx::{Pool, Sqlite};
use uuid::Uuid;

use crate::storage::error::DatabaseError;

#[derive(Debug)]
pub struct SqliteSenderKeyStore {
    database: Pool<Sqlite>,
}

impl SqliteSenderKeyStore {
    pub fn new(database: Pool<Sqlite>) -> Self {
        Self { database }
    }
}

#[async_trait(?Send)]
impl SenderKeyStore for SqliteSenderKeyStore {
    async fn store_sender_key(
        &mut self,
        sender: &ProtocolAddress,
        distribution_id: Uuid,
        record: &SenderKeyRecord,
    ) -> Result<(), SignalProtocolError> {
        let addr = format!("{}:{}", sender, distribution_id);
        let rec = BASE64_STANDARD.encode(record.serialize()?);

        sqlx::query!(
            r#"
            INSERT INTO DeviceSenderKeyStore (address, sender_key_record)
            VALUES (?, ?)
            ON CONFLICT(address) DO UPDATE SET sender_key_record = ?
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
                "store sender key",
                Box::new(DatabaseError::from(err)),
            )
        })
    }
    async fn load_sender_key(
        &mut self,
        sender: &ProtocolAddress,
        distribution_id: Uuid,
    ) -> Result<Option<SenderKeyRecord>, SignalProtocolError> {
        let addr = format!("{}:{}", sender, distribution_id);

        match sqlx::query!(
            r#"
            SELECT
                sender_key_record
            FROM
                DeviceSenderKeyStore
            WHERE
                address = ?
            "#,
            addr
        )
        .fetch_one(&self.database)
        .await
        {
            Ok(row) => SenderKeyRecord::deserialize(
                BASE64_STANDARD
                    .decode(row.sender_key_record)
                    .map_err(|err| {
                        SignalProtocolError::ApplicationCallbackError(
                            "load sender key",
                            Box::new(err),
                        )
                    })?
                    .as_slice(),
            )
            .map(Some),
            Err(_) => Ok(None), // TODO: Mistake?
        }
    }
}
