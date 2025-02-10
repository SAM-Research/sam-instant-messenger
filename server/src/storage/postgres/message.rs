use async_trait::async_trait;
use libsignal_protocol::ProtocolAddress;
use sam_common::sam_message::ServerEnvelope;

use crate::storage::{error::MessageStoreError, MessageStore};

#[derive(Debug)]
pub struct PostgresMessageStore {}

#[async_trait(?Send)]
impl MessageStore for PostgresMessageStore {
    async fn push_message_queue(
        &mut self,
        _address: &ProtocolAddress,
        _messages: Vec<ServerEnvelope>,
    ) -> Result<(), MessageStoreError> {
        todo!()
    }

    async fn pop_msg_queue(
        &mut self,
        _address: &ProtocolAddress,
    ) -> Result<Vec<ServerEnvelope>, MessageStoreError> {
        todo!()
    }

    async fn count_messages(&self, _address: &ProtocolAddress) -> Result<u32, MessageStoreError> {
        todo!()
    }

    async fn get_messages(
        &self,
        _address: &ProtocolAddress,
    ) -> Result<Vec<ServerEnvelope>, MessageStoreError> {
        todo!()
    }

    async fn delete_messages(
        &mut self,
        _address: &ProtocolAddress,
    ) -> Result<Vec<ServerEnvelope>, MessageStoreError> {
        todo!()
    }
}
