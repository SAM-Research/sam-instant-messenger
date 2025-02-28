use super::error::SamProtocolError;
use sam_common::sam_message::{ClientEnvelope, ServerEnvelope};
use tokio::sync::mpsc::Receiver;

#[async_trait::async_trait]
pub trait SamProtocolClient {
    async fn connect(&mut self) -> Result<Receiver<ServerEnvelope>, SamProtocolError>;
    async fn disconnect(&mut self) -> Result<(), SamProtocolError>;
    async fn is_connected(&self) -> bool;
    async fn send_message(&mut self, message: ClientEnvelope) -> Result<(), SamProtocolError>;
}
