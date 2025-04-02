use super::{decode::MessageStatus, error::ProtocolError};
use async_trait::async_trait;

use sam_common::{
    sam_message::{ClientEnvelope, ServerEnvelope},
    AccountId, DeviceId,
};
use tokio::sync::mpsc::Receiver;

#[async_trait::async_trait]
pub trait SamProtocolClient {
    async fn connect(&mut self) -> Result<Receiver<ServerEnvelope>, ProtocolError>;
    async fn disconnect(&mut self) -> Result<(), ProtocolError>;
    async fn is_connected(&self) -> bool;
    async fn send_message(
        &mut self,
        message: ClientEnvelope,
    ) -> Result<MessageStatus, ProtocolError>;
}

#[async_trait(?Send)]
pub trait ProtocolConfig {
    type ProtocolClient: SamProtocolClient;

    async fn create(
        self,
        account_id: AccountId,
        device_id: DeviceId,
        password: String,
    ) -> Result<Self::ProtocolClient, ProtocolError>;
}
