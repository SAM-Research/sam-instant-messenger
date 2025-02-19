use std::{collections::HashMap, sync::Arc};

use sam_common::{
    address::{AccountId, DeviceAddress, DeviceId, MessageId},
    sam_message::ServerEnvelope,
};
use tokio::sync::{mpsc, Mutex};

use crate::{managers::traits::message_manager::MessageManager, ServerError};

#[derive(Clone)]
pub struct InMemoryMessageManager {
    messages: Arc<Mutex<HashMap<DeviceAddress, HashMap<MessageId, ServerEnvelope>>>>,
    subscribers: Arc<Mutex<HashMap<DeviceAddress, mpsc::Sender<MessageId>>>>,
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
        }
    }
}

#[async_trait::async_trait]
impl MessageManager for InMemoryMessageManager {
    async fn insert_message(
        &mut self,
        account_id: AccountId,
        device_id: DeviceId,
        message_id: MessageId,
        message: ServerEnvelope,
    ) -> Result<(), ServerError> {
        let key = DeviceAddress::new(account_id, device_id);

        self.messages.lock().await.entry(key).or_default();

        let mut messages = self.messages.lock().await;
        let msgs = messages.get_mut(&key);

        if msgs
            .as_ref()
            .is_some_and(|map| map.contains_key(&message_id))
        {
            return Err(ServerError::EnvelopeExists);
        };

        let _ = msgs.and_then(|map| map.insert(message_id, message));

        Ok(())
    }

    async fn get_message(
        &self,
        account_id: AccountId,
        device_id: DeviceId,
        message_id: MessageId,
    ) -> Result<ServerEnvelope, ServerError> {
        let key = DeviceAddress::new(account_id, device_id);

        match self.messages.lock().await.get(&key) {
            Some(msgs) => msgs
                .get(&message_id)
                .cloned()
                .ok_or(ServerError::EnvelopeNotExists),
            None => Err(ServerError::AccountNotExist),
        }
    }

    async fn remove_message(
        &mut self,
        account_id: AccountId,
        device_id: DeviceId,
        message_id: MessageId,
    ) -> Result<(), ServerError> {
        let key = DeviceAddress::new(account_id, device_id);

        match self.messages.lock().await.get_mut(&key) {
            Some(msgs) => msgs
                .remove(&message_id)
                .ok_or(ServerError::EnvelopeNotExists)
                .map(|_| ()),
            None => Err(ServerError::AccountNotExist),
        }
    }

    async fn get_message_ids(
        &self,
        account_id: AccountId,
        device_id: DeviceId,
    ) -> Result<Vec<MessageId>, ServerError> {
        let key = DeviceAddress::new(account_id, device_id);

        self.messages
            .lock()
            .await
            .get(&key)
            .ok_or(ServerError::AccountNotExist)
            .map(|msgs| msgs.keys().cloned().collect::<Vec<MessageId>>())
    }

    async fn subscribe(
        &mut self,
        account_id: AccountId,
        device_id: DeviceId,
    ) -> Result<mpsc::Receiver<MessageId>, ServerError> {
        let key = DeviceAddress::new(account_id, device_id);
        let (sender, receiver) = mpsc::channel(10);

        if self.subscribers.lock().await.contains_key(&key) {
            return Err(ServerError::MessageSubscriberExists);
        }

        let _ = self.subscribers.lock().await.insert(key, sender);
        Ok(receiver)
    }

    async fn unsubscribe(
        &mut self,
        account_id: AccountId,
        device_id: DeviceId,
    ) -> Result<(), ServerError> {
        let key = DeviceAddress::new(account_id, device_id);

        if self.subscribers.lock().await.contains_key(&key) {
            return Err(ServerError::MessageSubscriberNotExists);
        }

        self.subscribers.lock().await.remove(&key);
        Ok(())
    }
}
