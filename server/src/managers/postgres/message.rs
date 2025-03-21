use sam_common::{sam_message::ServerEnvelope, AccountId, DeviceId};
use tokio::sync::mpsc;

use crate::{
    managers::traits::message_manager::{EnvelopeId, MessageManager},
    ServerError,
};

#[derive(Debug, Clone)]
pub struct PostgresMessageManager {}

#[async_trait::async_trait]
impl MessageManager for PostgresMessageManager {
    async fn channel_buffer(&self) -> usize {
        todo!()
    }

    async fn insert_envelope(
        &mut self,
        _account_id: AccountId,
        _device_id: DeviceId,
        _envelope_id: EnvelopeId,
        _envelope: ServerEnvelope,
    ) -> Result<(), ServerError> {
        todo!()
    }

    async fn get_envelope(
        &self,
        _account_id: AccountId,
        _device_id: DeviceId,
        _envelope_id: EnvelopeId,
    ) -> Result<ServerEnvelope, ServerError> {
        todo!()
    }

    async fn remove_envelope(
        &mut self,
        _account_id: AccountId,
        _device_id: DeviceId,
        _envelope_id: EnvelopeId,
    ) -> Result<(), ServerError> {
        todo!()
    }

    async fn get_envelope_ids(
        &self,
        _account_id: AccountId,
        _device_id: DeviceId,
    ) -> Option<Vec<EnvelopeId>> {
        todo!()
    }

    async fn subscribe(
        &mut self,
        _account_id: AccountId,
        _device_id: DeviceId,
    ) -> Result<mpsc::Receiver<EnvelopeId>, ServerError> {
        todo!()
    }

    async fn dispatch_envelopes(
        &mut self,
        _account_id: AccountId,
        _device_id: DeviceId,
    ) -> Result<(), ServerError> {
        todo!()
    }

    async fn unsubscribe(&mut self, _account_id: AccountId, _device_id: DeviceId) {
        todo!()
    }

    async fn add_pending_message(
        &mut self,
        _account_id: AccountId,
        _device_id: DeviceId,
        _envelope_id: EnvelopeId,
    ) -> Result<(), ServerError> {
        todo!()
    }

    async fn remove_pending_message(
        &mut self,
        _account_id: AccountId,
        _device_id: DeviceId,
        _envelope_id: EnvelopeId,
    ) -> Result<(), ServerError> {
        todo!()
    }
}
