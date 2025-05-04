use async_trait::async_trait;
use log::debug;
use sqlx::{Pool, Sqlite};
use tokio::sync::broadcast::{self, Receiver, Sender};

use crate::{
    encryption::envelope::DecryptedEnvelope,
    storage::{
        error::{DatabaseError, MessageStoreError},
        traits::message::MessageStore,
    },
};

pub struct SqliteMessageStore {
    database: Pool<Sqlite>,
    sender: Sender<DecryptedEnvelope>,
}

impl SqliteMessageStore {
    pub fn new(database: Pool<Sqlite>, buffer: usize) -> Self {
        let (sender, _) = broadcast::channel(buffer);
        Self { database, sender }
    }
}

#[async_trait(?Send)]
impl MessageStore for SqliteMessageStore {
    async fn store_message(
        &mut self,
        envelope: DecryptedEnvelope,
    ) -> Result<(), MessageStoreError> {
        let account_id = envelope.source_account_id().to_string();
        let device_id = envelope.source_device_id().to_string();
        let content = envelope.content_bytes();
        let res = sqlx::query!(
            r#"
            INSERT INTO MessageStore (contact_id, content)
            VALUES (
            (SELECT id FROM Contacts WHERE account_id = ? AND device_id = ?),
            ?
            );
            "#,
            account_id,
            device_id,
            content
        )
        .execute(&self.database)
        .await
        .map(|_| ())
        .inspect_err(|e| debug!("{e}"))
        .map_err(|err| DatabaseError::Database(format!("{err}")));
        let send_res = match res {
            Ok(()) => self.sender.send(envelope),
            Err(e) => Err(e)?,
        };
        if let Err(e) = send_res {
            debug!("No receivers on broadcast channel: {e}");
        }
        Ok(())
    }
    fn subscribe(&self) -> Receiver<DecryptedEnvelope> {
        self.sender.subscribe()
    }
}

#[cfg(test)]
mod test {
    use std::time::Duration;

    use libsignal_protocol::IdentityKeyPair;
    use rand::rngs::OsRng;
    use sam_common::{address::RegistrationId, AccountId, DeviceId};

    use crate::{
        encryption::envelope::DecryptedEnvelope,
        storage::{
            sqlite::SqliteStoreConfig, traits::message::MessageStore, ContactStore, StoreConfig,
        },
    };

    #[tokio::test]
    async fn test_store_and_send_to_subscriber() {
        let mut csprng = OsRng;
        let mut store = SqliteStoreConfig::in_memory(10)
            .await
            .expect("can create inmemory")
            .create_store(
                IdentityKeyPair::generate(&mut csprng),
                RegistrationId::generate(&mut csprng),
            )
            .await
            .expect("Can create store");

        let mut listener = store.message_store.subscribe();

        let account_id = AccountId::generate();
        let device_id: DeviceId = 1.into();
        store
            .contact_store
            .add_device(account_id, device_id)
            .await
            .expect("Can add device");
        store
            .message_store
            .store_message(
                DecryptedEnvelope::builder()
                    .content(vec![55, 66, 77])
                    .source_account_id(account_id)
                    .source_device_id(device_id)
                    .build(),
            )
            .await
            .expect("Can store message");

        let timeout = tokio::time::timeout(Duration::from_millis(100), listener.recv()).await;
        let envelope = timeout
            .expect("Sender does not timeout")
            .expect("Sender sends message");

        assert!(*envelope.content_bytes() == vec![55, 66, 77]);
        assert!(envelope.source_account_id() == account_id);
        assert!(envelope.source_device_id() == device_id);
    }

    #[tokio::test]
    async fn message_sender_persists_after_dead_receiver() {
        let _ = env_logger::try_init();
        let mut csprng = OsRng;
        let mut store = SqliteStoreConfig::in_memory(10)
            .await
            .expect("can create inmemory")
            .create_store(
                IdentityKeyPair::generate(&mut csprng),
                RegistrationId::generate(&mut csprng),
            )
            .await
            .expect("Can create store");

        {
            let _ = store.message_store.subscribe();
        }

        let account_id = AccountId::generate();
        let device_id: DeviceId = 1.into();
        store
            .contact_store
            .add_device(account_id, device_id)
            .await
            .expect("Can add device");
        store
            .message_store
            .store_message(
                DecryptedEnvelope::builder()
                    .content(vec![55, 66, 77])
                    .source_account_id(account_id)
                    .source_device_id(device_id)
                    .build(),
            )
            .await
            .expect("Can store message");
    }
}
