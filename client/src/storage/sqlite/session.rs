use async_trait::async_trait;
use base64::{prelude::BASE64_STANDARD, Engine as _};
use libsignal_protocol::{ProtocolAddress, SenderKeyRecord, SenderKeyStore, SignalProtocolError};
use sqlx::{Pool, Sqlite};
use uuid::Uuid;

use crate::ClientError;

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
                Box::new(ClientError::from(err)),
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

#[cfg(test)]
mod test {
    use libsignal_protocol::SessionRecord;
    use libsignal_protocol::SessionStore;

    use crate::storage::sqlite::{
        sender_key::SqliteSessionStore,
        sqlite_test::{alice_address, connect},
    };

    #[tokio::test]
    async fn load_and_store_session() {
        let mut session_store = SqliteSessionStore::new(connect().await);
        let address = alice_address();
        let record = SessionRecord::new_fresh();
        session_store
            .store_session(&address, &record)
            .await
            .unwrap();

        assert_eq!(
            session_store
                .load_session(&address)
                .await
                .unwrap()
                .unwrap()
                .serialize()
                .unwrap(),
            record.serialize().unwrap()
        );
    }

    #[tokio::test]
    async fn no_session_in_new_store() {
        let session_store = SqliteSessionStore::new(connect().await);
        let address = alice_address();

        assert!(session_store
            .load_session(&address)
            .await
            .unwrap()
            .is_none());
    }
}
