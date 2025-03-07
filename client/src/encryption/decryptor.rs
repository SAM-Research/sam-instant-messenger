use sam_common::sam_message::ServerEnvelope;
use tokio::sync::mpsc::Receiver;

use crate::storage::{Store, StoreType};

use super::encrypt::decrypt;

pub async fn decryptor(mut receiver: Receiver<ServerEnvelope>, mut store: Store<impl StoreType>) {
    while let Some(envelope) = receiver.recv().await {
        let msg = decrypt(envelope, &mut store).await;
    }
}
