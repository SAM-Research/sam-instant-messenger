use crate::ServerError;
use sam_common::{
    address::{AccountId, DeviceId, MessageId},
    sam_message::ServerEnvelope,
};
use tokio::sync::mpsc::Receiver;

pub type EnvelopeId = MessageId;

#[async_trait::async_trait]
pub trait MessageManager: Send + Sync + Clone {
    async fn insert_envelope(
        &mut self,
        account_id: AccountId,
        device_id: DeviceId,
        envelope_id: EnvelopeId,
        envelope: ServerEnvelope,
    ) -> Result<(), ServerError>;
    async fn get_envelope(
        &self,
        account_id: AccountId,
        device_id: DeviceId,
        envelope_id: EnvelopeId,
    ) -> Result<ServerEnvelope, ServerError>;
    async fn remove_envelope(
        &mut self,
        account_id: AccountId,
        device_id: DeviceId,
        envelope_id: EnvelopeId,
    ) -> Result<(), ServerError>;
    async fn get_envelope_ids(
        &self,
        account_id: AccountId,
        device_id: DeviceId,
    ) -> Result<Vec<EnvelopeId>, ServerError>;
    async fn subscribe(
        &mut self,
        account_id: AccountId,
        device_id: DeviceId,
    ) -> Result<Receiver<EnvelopeId>, ServerError>;
    async fn unsubscribe(&mut self, account_id: AccountId, device_id: DeviceId);
    async fn add_pending_message(
        &mut self,
        account_id: AccountId,
        device_id: DeviceId,
        envelope_id: EnvelopeId,
    ) -> Result<(), ServerError>;
    async fn remove_pending_message(
        &mut self,
        account_id: AccountId,
        device_id: DeviceId,
        envelope_id: EnvelopeId,
    ) -> Result<(), ServerError>;
}
