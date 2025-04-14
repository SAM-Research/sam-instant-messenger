use async_trait::async_trait;
use tokio::sync::broadcast::Receiver;

use crate::{encryption::envelope::DecryptedEnvelope, storage::error::MessageStoreError};

#[async_trait(?Send)]
pub trait MessageStore {
    async fn store_message(&mut self, envelope: DecryptedEnvelope)
        -> Result<(), MessageStoreError>;
    fn subscribe(&self) -> Receiver<DecryptedEnvelope>;
}
