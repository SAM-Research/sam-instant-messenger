use async_trait::async_trait;
use sam_common::{address::DeviceAddress, sam_message::ServerEnvelope};

use crate::storage::{error::MessageStoreError, traits::MessageStore};

#[derive(Debug)]
pub struct InMemoryMessageStore {}

#[async_trait(?Send)]
impl MessageStore for InMemoryMessageStore {
    async fn push_message_queue(
        &mut self,
        _messages: Vec<ServerEnvelope>,
        _address: &DeviceAddress,
    ) -> Result<(), MessageStoreError> {
        todo!()
    }

    async fn pop_msg_queue(
        &mut self,
        _address: &DeviceAddress,
    ) -> Result<Vec<ServerEnvelope>, MessageStoreError> {
        todo!()
    }

    async fn count_messages(&self, _address: &DeviceAddress) -> Result<u32, MessageStoreError> {
        todo!()
    }

    async fn get_messages(
        &self,
        _address: &DeviceAddress,
    ) -> Result<Vec<ServerEnvelope>, MessageStoreError> {
        todo!()
    }

    async fn delete_messages(
        &mut self,
        _address: &DeviceAddress,
    ) -> Result<Vec<ServerEnvelope>, MessageStoreError> {
        todo!()
    }
}
