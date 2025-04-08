use async_trait::async_trait;
use sam_common::{sam_message::ServerEnvelope, AccountId, DeviceId};
use sqlx::{Pool, Postgres};
use tokio::sync::mpsc;

use crate::managers::{
    error::MessageManagerError,
    traits::message_manager::{EnvelopeId, MessageManager},
};

#[derive(Debug, Clone)]
pub struct PostgresMessageManager {
    pool: Pool<Postgres>,
}

impl PostgresMessageManager {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
}

#[async_trait]
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
    ) -> Result<(), MessageManagerError> {
        todo!()
    }

    async fn get_envelope(
        &self,
        _account_id: AccountId,
        _device_id: DeviceId,
        _envelope_id: EnvelopeId,
    ) -> Result<ServerEnvelope, MessageManagerError> {
        todo!()
    }

    async fn remove_envelope(
        &mut self,
        _account_id: AccountId,
        _device_id: DeviceId,
        _envelope_id: EnvelopeId,
    ) -> Result<(), MessageManagerError> {
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
    ) -> Result<mpsc::Receiver<EnvelopeId>, MessageManagerError> {
        todo!()
    }

    async fn dispatch_envelopes(
        &mut self,
        _account_id: AccountId,
        _device_id: DeviceId,
    ) -> Result<(), MessageManagerError> {
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
    ) -> Result<(), MessageManagerError> {
        todo!()
    }

    async fn remove_pending_message(
        &mut self,
        _account_id: AccountId,
        _device_id: DeviceId,
        _envelope_id: EnvelopeId,
    ) -> Result<(), MessageManagerError> {
        todo!()
    }
}
