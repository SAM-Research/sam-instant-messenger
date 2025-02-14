use crate::ServerError;
use sam_common::sam_message::ServerEnvelope;
use tokio::sync::mpsc::Receiver;
use uuid::Uuid;

#[async_trait::async_trait]
pub trait MessageManager: Send {
    async fn insert_message(
        &mut self,
        account_id: &Uuid,
        device_id: &u32,
        message_id: &Uuid,
        message: ServerEnvelope,
    ) -> Result<(), ServerError>;
    async fn get_message(
        &self,
        account_id: &Uuid,
        device_id: &u32,
        message_id: &Uuid,
    ) -> Result<ServerEnvelope, ServerError>;
    async fn remove_message(
        &mut self,
        account_id: &Uuid,
        device_id: &u32,
        message_id: &Uuid,
    ) -> Result<(), ServerError>;
    async fn get_messages(
        &self,
        account_id: &Uuid,
        device_id: &u32,
    ) -> Result<Vec<Uuid>, ServerError>;
    async fn subscribe(
        &mut self,
        account_id: &Uuid,
        device_id: &u32,
    ) -> Result<Receiver<Uuid>, ServerError>;
    async fn unsubscribe(&mut self, account_id: &Uuid, device_id: &u32) -> Result<(), ServerError>;
}
