use async_trait::async_trait;
use sam_common::{address::DeviceAddress, ValidServerEnvelope};

use crate::storage::{error::MessageStoreError, traits::MessageStore};

#[derive(Debug)]
pub struct PostgresMessageStore {}

#[async_trait(?Send)]
impl MessageStore for PostgresMessageStore {
    async fn push_message_queue(
        &mut self,
        _messages: Vec<ValidServerEnvelope>,
    ) -> Result<(), MessageStoreError> {
        todo!()
    }

    async fn pop_msg_queue(
        &mut self,
        _address: &DeviceAddress,
    ) -> Result<Vec<ValidServerEnvelope>, MessageStoreError> {
        todo!()
    }

    async fn count_messages(&self, _address: &DeviceAddress) -> Result<usize, MessageStoreError> {
        todo!()
    }

    async fn get_messages(
        &self,
        _address: &DeviceAddress,
    ) -> Result<Vec<ValidServerEnvelope>, MessageStoreError> {
        todo!()
    }

    async fn clear_message_queue(
        &mut self,
        _address: &DeviceAddress,
    ) -> Result<(), MessageStoreError> {
        todo!()
    }
}
