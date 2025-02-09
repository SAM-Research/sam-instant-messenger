use crate::ServerError;
use libsignal_protocol::ProtocolAddress;
use sam_common::sam_message::{ClientEnvelope, ServerEnvelope};
use tokio::sync::broadcast::Receiver;
use uuid::Uuid;

#[async_trait::async_trait]
pub trait MessageManager: Send {
    async fn insert_message(
        &mut self,
        addr: ProtocolAddress,
        envelope: ClientEnvelope,
    ) -> Result<(), ServerError>;
    async fn get_message(
        &mut self,
        addr: ProtocolAddress,
        envelope_id: Uuid,
    ) -> Result<ServerEnvelope, ServerError>;
    async fn remove_message(
        &mut self,
        addr: ProtocolAddress,
        envelope_id: Uuid,
    ) -> Result<(), ServerError>;
    async fn subscribe(&self) -> Receiver<Uuid>;
}
