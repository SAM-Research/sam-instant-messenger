use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use sam_common::sam_message::ServerEnvelope;
use tokio::sync::{mpsc, Mutex};
use uuid::Uuid;

use crate::{managers::traits::message_manager::MessageManager, ServerError};

use super::device_key;

#[derive(Clone)]
pub struct InMemoryMessageManager {
    messages: Arc<Mutex<HashMap<String, HashMap<Uuid, ServerEnvelope>>>>,
    subscribers: Arc<Mutex<HashMap<String, mpsc::Sender<Uuid>>>>,
    pending_messages: Arc<Mutex<HashSet<String>>>,
}

impl Default for InMemoryMessageManager {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryMessageManager {
    pub fn new() -> Self {
        InMemoryMessageManager {
            messages: Arc::new(Mutex::new(HashMap::new())),
            subscribers: Arc::new(Mutex::new(HashMap::new())),
            pending_messages: Arc::new(Mutex::new(HashSet::new())),
        }
    }
}

#[async_trait::async_trait]
impl MessageManager for InMemoryMessageManager {
    async fn insert_envelope(
        &mut self,
        account_id: Uuid,
        device_id: u32,
        envelope_id: Uuid,
        envelope: ServerEnvelope,
    ) -> Result<(), ServerError> {
        let key = device_key(account_id, device_id);

        if !self.messages.lock().await.contains_key(&key) {
            self.messages
                .lock()
                .await
                .insert(key.clone(), HashMap::new());
        }

        let mut messages = self.messages.lock().await;
        let msgs = messages.get_mut(&key);

        if msgs
            .as_ref()
            .is_some_and(|map| map.contains_key(&envelope_id))
        {
            return Err(ServerError::EnvelopeExists);
        };

        let _ = msgs.and_then(|map| map.insert(envelope_id, envelope));

        Ok(())
    }

    async fn get_envelope(
        &self,
        account_id: Uuid,
        device_id: u32,
        envelope_id: Uuid,
    ) -> Result<ServerEnvelope, ServerError> {
        let key = device_key(account_id, device_id);

        match self.messages.lock().await.get(&key) {
            Some(msgs) => msgs
                .get(&envelope_id)
                .cloned()
                .ok_or(ServerError::EnvelopeNotExists),
            None => Err(ServerError::AccountNotExist),
        }
    }

    async fn remove_envelope(
        &mut self,
        account_id: Uuid,
        device_id: u32,
        envelope_id: Uuid,
    ) -> Result<(), ServerError> {
        let key = device_key(account_id, device_id);

        match self.messages.lock().await.get_mut(&key) {
            Some(msgs) => msgs
                .remove(&envelope_id)
                .ok_or(ServerError::EnvelopeNotExists)
                .map(|_| ()),
            None => Err(ServerError::AccountNotExist),
        }
    }

    async fn get_envelope_ids(
        &self,
        account_id: Uuid,
        device_id: u32,
    ) -> Result<Vec<Uuid>, ServerError> {
        let key = device_key(account_id, device_id);

        self.messages
            .lock()
            .await
            .get(&key)
            .ok_or(ServerError::AccountNotExist)
            .map(|msgs| msgs.keys().cloned().collect::<Vec<Uuid>>())
    }

    async fn subscribe(
        &mut self,
        account_id: Uuid,
        device_id: u32,
    ) -> Result<mpsc::Receiver<Uuid>, ServerError> {
        let key = device_key(account_id, device_id);
        let (sender, receiver) = mpsc::channel(10);

        if self.subscribers.lock().await.contains_key(&key) {
            return Err(ServerError::MessageSubscriberExists);
        }

        let _ = self.subscribers.lock().await.insert(key, sender);
        Ok(receiver)
    }

    async fn unsubscribe(&mut self, account_id: Uuid, device_id: u32) {
        let key = device_key(account_id, device_id);

        if self.subscribers.lock().await.contains_key(&key) {
            return;
        }

        self.subscribers.lock().await.remove(&key);
    }

    async fn add_pending_message(
        &mut self,
        account_id: Uuid,
        device_id: u32,
        message_id: Uuid,
    ) -> Result<(), ServerError> {
        let key = message_key(account_id, device_id, message_id);

        if self.pending_messages.lock().await.contains(&key) {
            return Err(ServerError::MessageAlreadyPending);
        }
        self.pending_messages.lock().await.insert(key);
        Ok(())
    }

    async fn remove_pending_message(
        &mut self,
        account_id: Uuid,
        device_id: u32,
        message_id: Uuid,
    ) -> Result<(), ServerError> {
        let key = message_key(account_id, device_id, message_id);

        if !self.pending_messages.lock().await.contains(&key) {
            return Err(ServerError::MessageNotPending);
        }
        self.pending_messages.lock().await.remove(&key);
        Ok(())
    }
}

fn message_key(account_id: Uuid, device_id: u32, message_id: Uuid) -> String {
    format!("{}.{}", device_key(account_id, device_id), message_id)
}
