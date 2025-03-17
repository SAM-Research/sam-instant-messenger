use async_trait::async_trait;
use log::debug;
use tokio::sync::broadcast::{self, Receiver, Sender};

use crate::{
    encryption::envelope::DecryptedEnvelope, storage::traits::message::MessageStore, ClientError,
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
    async fn store_message(&mut self, envelope: DecryptedEnvelope) -> Result<(), ClientError> {
        self.messages.push(envelope.clone());
        self.sender
            .send(envelope)
            .inspect_err(|e| debug!("{e}"))
            .map_err(|_| ClientError::SendError)
            .map(|_| ())
    }
    fn subscribe(&self) -> Receiver<DecryptedEnvelope> {
        self.sender.subscribe()
    }
}
