use std::collections::HashMap;

use async_trait::async_trait;
use sam_common::{address::DeviceAddress, ValidServerEnvelope};

use crate::storage::{error::MessageStoreError, traits::MessageStore};

#[derive(Debug)]
pub struct InMemoryMessageStore {
    messages: HashMap<DeviceAddress, Vec<ValidServerEnvelope>>,
}

#[async_trait(?Send)]
impl MessageStore for InMemoryMessageStore {
    async fn push_message_queue(
        &mut self,
        messages: Vec<ValidServerEnvelope>,
    ) -> Result<(), MessageStoreError> {
        messages.into_iter().for_each(|message| {
            self.messages
                .entry(message.destination())
                .or_default()
                .push(message);
        });
        Ok(())
    }

    async fn pop_msg_queue(
        &mut self,
        address: &DeviceAddress,
    ) -> Result<Vec<ValidServerEnvelope>, MessageStoreError> {
        Ok(self.messages.remove(address).unwrap_or_default())
    }

    async fn count_messages(&self, address: &DeviceAddress) -> Result<usize, MessageStoreError> {
        Ok(self.messages.get(address).map_or(0, |queue| queue.len()))
    }

    async fn get_messages(
        &self,
        address: &DeviceAddress,
    ) -> Result<Vec<ValidServerEnvelope>, MessageStoreError> {
        Ok(self.messages.get(address).cloned().unwrap_or_default())
    }

    async fn clear_message_queue(
        &mut self,
        address: &DeviceAddress,
    ) -> Result<(), MessageStoreError> {
        if let Some(queue) = self.messages.get_mut(address) {
            queue.clear()
        }
        Ok(())
    }
}
