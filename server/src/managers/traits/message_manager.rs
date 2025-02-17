use crate::ServerError;
use sam_common::sam_message::ServerEnvelope;
use tokio::sync::mpsc::Receiver;
use uuid::Uuid;

#[async_trait::async_trait]
pub trait MessageManager: Send + Sync + Clone {
    async fn insert_envelope(
        &mut self,
        account_id: Uuid,
        device_id: u32,
        envelope_id: Uuid,
        envelope: ServerEnvelope,
    ) -> Result<(), ServerError>;
    async fn get_envelope(
        &self,
        account_id: Uuid,
        device_id: u32,
        envelope_id: Uuid,
    ) -> Result<ServerEnvelope, ServerError>;
    async fn remove_envelope(
        &mut self,
        account_id: Uuid,
        device_id: u32,
        envelope_id: Uuid,
    ) -> Result<(), ServerError>;
    async fn get_envelope_ids(
        &self,
        account_id: Uuid,
        device_id: u32,
    ) -> Result<Vec<Uuid>, ServerError>;
    async fn subscribe(
        &mut self,
        account_id: Uuid,
        device_id: u32,
    ) -> Result<Receiver<Uuid>, ServerError>;
    async fn unsubscribe(&mut self, account_id: Uuid, device_id: u32);
    async fn add_pending_message(
        &mut self,
        account_id: Uuid,
        device_id: u32,
        message_id: Uuid,
    ) -> Result<(), ServerError>;
    async fn remove_pending_message(
        &mut self,
        account_id: Uuid,
        device_id: u32,
        message_id: Uuid,
    ) -> Result<(), ServerError>;
}
