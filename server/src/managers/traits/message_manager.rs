use crate::managers::error::MessageManagerError;
use async_trait::async_trait;
use sam_common::{
    address::{AccountId, DeviceId, MessageId},
    sam_message::ServerEnvelope,
};
use tokio::sync::mpsc::Receiver;

pub type EnvelopeId = MessageId;

#[async_trait]
pub trait MessageManager: Send + Sync + Clone {
    async fn channel_buffer(&self) -> usize;
    async fn insert_envelope(
        &mut self,
        account_id: AccountId,
        device_id: DeviceId,
        envelope_id: EnvelopeId,
        envelope: ServerEnvelope,
    ) -> Result<(), MessageManagerError>;
    async fn get_envelope(
        &self,
        account_id: AccountId,
        device_id: DeviceId,
        envelope_id: EnvelopeId,
    ) -> Result<ServerEnvelope, MessageManagerError>;
    async fn remove_envelope(
        &mut self,
        account_id: AccountId,
        device_id: DeviceId,
        envelope_id: EnvelopeId,
    ) -> Result<(), MessageManagerError>;
    async fn get_envelope_ids(
        &self,
        account_id: AccountId,
        device_id: DeviceId,
    ) -> Option<Vec<EnvelopeId>>;
    async fn subscribe(
        &mut self,
        account_id: AccountId,
        device_id: DeviceId,
    ) -> Result<Receiver<EnvelopeId>, MessageManagerError>;
    async fn dispatch_envelopes(
        &mut self,
        account_id: AccountId,
        device_id: DeviceId,
    ) -> Result<(), MessageManagerError>;
    async fn unsubscribe(&mut self, account_id: AccountId, device_id: DeviceId);
    async fn add_pending_message(
        &mut self,
        account_id: AccountId,
        device_id: DeviceId,
        envelope_id: EnvelopeId,
    ) -> Result<(), MessageManagerError>;
    async fn remove_pending_message(
        &mut self,
        account_id: AccountId,
        device_id: DeviceId,
        envelope_id: EnvelopeId,
    ) -> Result<(), MessageManagerError>;
}
