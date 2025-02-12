use sam_common::sam_message::{ClientEnvelope, ServerEnvelope};
use tokio::sync::broadcast::Receiver;
use uuid::Uuid;

use crate::{managers::traits::message_manager::MessageManager, ServerError};

pub struct InMemoryMessageManager {}

#[async_trait::async_trait]
impl MessageManager for InMemoryMessageManager {
    async fn insert_message(
        &mut self,
        account_id: &Uuid,
        device_id: &u32,
        message: ClientEnvelope,
    ) -> Result<(), ServerError> {
        unimplemented!()
    }

    async fn get_message(
        &self,
        account_id: &Uuid,
        device_id: &u32,
        message_id: Uuid,
    ) -> Result<ServerEnvelope, ServerError> {
        unimplemented!()
    }

    async fn remove_message(
        &mut self,
        account_id: &Uuid,
        device_id: &u32,
        message_id: Uuid,
    ) -> Result<(), ServerError> {
        unimplemented!()
    }

    async fn get_messages(
        &self,
        account_id: &Uuid,
        device_id: &u32,
    ) -> Result<Vec<Uuid>, ServerError> {
        unimplemented!()
    }

    async fn subscribe(&self, account_id: &Uuid, device_id: &u32) -> Receiver<Uuid> {
        unimplemented!()
    }
}
