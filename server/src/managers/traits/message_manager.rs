use crate::ServerError;
use sam_common::{
    address::{AccountId, DeviceId, MessageId},
    sam_message::ServerEnvelope,
};
use tokio::sync::mpsc::Receiver;

#[async_trait::async_trait]
pub trait MessageManager: Send {
    async fn insert_message(
        &mut self,
        account_id: AccountId,
        device_id: DeviceId,
        message_id: MessageId,
        message: ServerEnvelope,
    ) -> Result<(), ServerError>;
    async fn get_message(
        &self,
        account_id: AccountId,
        device_id: DeviceId,
        message_id: MessageId,
    ) -> Result<ServerEnvelope, ServerError>;
    async fn remove_message(
        &mut self,
        account_id: AccountId,
        device_id: DeviceId,
        message_id: MessageId,
    ) -> Result<(), ServerError>;
    async fn get_messages(
        &self,
        account_id: AccountId,
        device_id: DeviceId,
    ) -> Result<Vec<MessageId>, ServerError>;
    async fn subscribe(
        &mut self,
        account_id: AccountId,
        device_id: DeviceId,
    ) -> Result<Receiver<MessageId>, ServerError>;
    async fn unsubscribe(
        &mut self,
        account_id: AccountId,
        device_id: DeviceId,
    ) -> Result<(), ServerError>;
}
