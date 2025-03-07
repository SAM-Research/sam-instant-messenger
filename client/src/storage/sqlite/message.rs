use async_trait::async_trait;
use sqlx::{Pool, Sqlite};
use tokio::sync::broadcast::{Receiver, Sender};

use crate::{
    encryption::envelope::DecryptedEnvelope, storage::traits::message::MessageStore, ClientError,
};

struct SqliteMessageStore {
    database: Pool<Sqlite>,
    sender: Sender<DecryptedEnvelope>,
}

#[async_trait(?Send)]
impl MessageStore for SqliteMessageStore {
    async fn store_message(&mut self, envelope: DecryptedEnvelope) -> Result<(), ClientError> {
        let account_id = envelope.source_account_id().to_string();
        let device_id = envelope.source_device_id().to_string();
        let content = envelope.content_bytes();
        let x = sqlx::query!(
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
        .map_err(|err| ClientError::Database(format!("{err}")));
        //self.sender.send(envelope);
        todo!()
    }
    async fn subscribe(&self) -> Receiver<DecryptedEnvelope> {
        self.sender.subscribe()
    }
}
