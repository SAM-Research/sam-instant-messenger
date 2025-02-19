use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use sam_common::{
    address::{AccountId, DeviceAddress, DeviceId},
    sam_message::ServerEnvelope,
};
use tokio::sync::{mpsc, Mutex};

use crate::{
    managers::traits::message_manager::{EnvelopeId, MessageManager},
    ServerError,
};

#[derive(Clone)]
pub struct InMemoryMessageManager {
    envelopes: Arc<Mutex<HashMap<DeviceAddress, HashMap<EnvelopeId, ServerEnvelope>>>>,
    subscribers: Arc<Mutex<HashMap<DeviceAddress, mpsc::Sender<EnvelopeId>>>>,
    pending_messages: Arc<Mutex<HashSet<EnvelopeKey>>>,
}

impl Default for InMemoryMessageManager {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryMessageManager {
    pub fn new() -> Self {
        InMemoryMessageManager {
            envelopes: Arc::new(Mutex::new(HashMap::new())),
            subscribers: Arc::new(Mutex::new(HashMap::new())),
            pending_messages: Arc::new(Mutex::new(HashSet::new())),
        }
    }
}

#[async_trait::async_trait]
impl MessageManager for InMemoryMessageManager {
    async fn insert_envelope(
        &mut self,
        account_id: AccountId,
        device_id: DeviceId,
        envelope_id: EnvelopeId,
        message: ServerEnvelope,
    ) -> Result<(), ServerError> {
        let key = DeviceAddress::new(account_id, device_id);

        self.envelopes.lock().await.entry(key).or_default();

        let mut messages = self.envelopes.lock().await;
        let msgs = messages.get_mut(&key);

        if msgs
            .as_ref()
            .is_some_and(|map| map.contains_key(&envelope_id))
        {
            return Err(ServerError::EnvelopeExists);
        };

        let _ = msgs.and_then(|map| map.insert(envelope_id, message));
        if let Some(sender) = self.subscribers.lock().await.get(&key) {
            sender
                .send(envelope_id)
                .await
                .map_err(|_| ServerError::MessageSubscriberSendErorr)?;
        }
        Ok(())
    }

    async fn get_envelope(
        &self,
        account_id: AccountId,
        device_id: DeviceId,
        envelope_id: EnvelopeId,
    ) -> Result<ServerEnvelope, ServerError> {
        let key = DeviceAddress::new(account_id, device_id);

        match self.envelopes.lock().await.get(&key) {
            Some(msgs) => msgs
                .get(&envelope_id)
                .cloned()
                .ok_or(ServerError::EnvelopeNotExists),
            None => Err(ServerError::AccountNotExist),
        }
    }

    async fn remove_envelope(
        &mut self,
        account_id: AccountId,
        device_id: DeviceId,
        envelope_id: EnvelopeId,
    ) -> Result<(), ServerError> {
        let key = DeviceAddress::new(account_id, device_id);

        match self.envelopes.lock().await.get_mut(&key) {
            Some(msgs) => msgs
                .remove(&envelope_id)
                .ok_or(ServerError::EnvelopeNotExists)
                .map(|_| ()),
            None => Err(ServerError::AccountNotExist),
        }
    }

    async fn get_envelope_ids(
        &self,
        account_id: AccountId,
        device_id: DeviceId,
    ) -> Result<Vec<EnvelopeId>, ServerError> {
        let key = DeviceAddress::new(account_id, device_id);

        self.envelopes
            .lock()
            .await
            .get(&key)
            .ok_or(ServerError::AccountNotExist)
            .map(|msgs| msgs.keys().cloned().collect::<Vec<EnvelopeId>>())
    }

    async fn subscribe(
        &mut self,
        account_id: AccountId,
        device_id: DeviceId,
    ) -> Result<mpsc::Receiver<EnvelopeId>, ServerError> {
        let key = DeviceAddress::new(account_id, device_id);
        let (sender, receiver) = mpsc::channel(10);

        if self.subscribers.lock().await.contains_key(&key) {
            return Err(ServerError::MessageSubscriberExists);
        }

        let _ = self.subscribers.lock().await.insert(key, sender);
        Ok(receiver)
    }

    async fn unsubscribe(&mut self, account_id: AccountId, device_id: DeviceId) {
        let key = DeviceAddress::new(account_id, device_id);

        if self.subscribers.lock().await.contains_key(&key) {
            return;
        }

        self.subscribers.lock().await.remove(&key);
    }

    async fn add_pending_message(
        &mut self,
        account_id: AccountId,
        device_id: DeviceId,
        envelope_id: EnvelopeId,
    ) -> Result<(), ServerError> {
        let key = EnvelopeKey::new(account_id, device_id, envelope_id);

        if self.pending_messages.lock().await.contains(&key) {
            return Err(ServerError::MessageAlreadyPending);
        }
        self.pending_messages.lock().await.insert(key);
        Ok(())
    }

    async fn remove_pending_message(
        &mut self,
        account_id: AccountId,
        device_id: DeviceId,
        envelope_id: EnvelopeId,
    ) -> Result<(), ServerError> {
        let key = EnvelopeKey::new(account_id, device_id, envelope_id);

        if !self.pending_messages.lock().await.contains(&key) {
            return Err(ServerError::MessageNotPending);
        }
        self.pending_messages.lock().await.remove(&key);
        Ok(())
    }
}

#[derive(Hash, PartialEq, Eq)]
struct EnvelopeKey {
    account_id: AccountId,
    device_id: DeviceId,
    envelope_id: EnvelopeId,
}

impl EnvelopeKey {
    fn new(account_id: AccountId, device_id: DeviceId, envelope_id: EnvelopeId) -> Self {
        EnvelopeKey {
            account_id,
            device_id,
            envelope_id,
        }
    }
}
