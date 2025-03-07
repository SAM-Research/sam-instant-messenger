use log::error;
use sam_common::sam_message::ServerEnvelope;
use tokio::sync::mpsc::Receiver;

use crate::storage::{traits::message::MessageStore, Store, StoreType};

use super::encrypt::decrypt;

pub async fn decryptor(mut receiver: Receiver<ServerEnvelope>, mut store: Store<impl StoreType>) {
    while let Some(envelope) = receiver.recv().await {
        // TODO: How should we handle failure to decrypt and/or store message?
        let envelope = match decrypt(envelope, &mut store).await {
            Ok(denvelope) => denvelope,
            Err(e) => {
                error!("Failed to decrypt message {e}");
                continue;
            }
        };

        let _ = store
            .message_store
            .store_message(envelope)
            .await
            .inspect_err(|e| error!("Failed to store message {e}"));
    }
}
