use async_trait::async_trait;
use log::debug;
use tokio::sync::broadcast::{self, Receiver, Sender};

use crate::{
    encryption::envelope::DecryptedEnvelope,
    storage::{error::MessageStoreError, traits::message::MessageStore},
};

pub struct InMemoryMessageStore {
    messages: Vec<DecryptedEnvelope>,
    sender: Sender<DecryptedEnvelope>,
}

impl InMemoryMessageStore {
    pub fn new(buffer: usize) -> Self {
        let (sender, _) = broadcast::channel(buffer);
        Self {
            messages: Vec::new(),
            sender,
        }
    }
}

#[async_trait(?Send)]
impl MessageStore for InMemoryMessageStore {
    async fn store_message(
        &mut self,
        envelope: DecryptedEnvelope,
    ) -> Result<(), MessageStoreError> {
        self.messages.push(envelope.clone());
        if let Err(e) = self.sender.send(envelope).inspect_err(|e| debug!("{e}")) {
            debug!("No receivers on broadcast channel: {e}");
        }
        Ok(())
    }
    fn subscribe(&self) -> Receiver<DecryptedEnvelope> {
        self.sender.subscribe()
    }
}
