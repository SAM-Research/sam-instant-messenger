use libsignal_protocol::ProtocolAddress;
use sam_common::sam_message::ServerEnvelope;

#[async_trait::async_trait]
pub trait MessageCache {
    async fn insert(
        &self,
        address: &ProtocolAddress,
        envelope: &mut ServerEnvelope,
        message_guid: &str,
    ) -> String;
    async fn remove(
        &self,
        address: &ProtocolAddress,
        message_guids: Vec<String>,
    ) -> Vec<ServerEnvelope>;
    async fn has_messages(&self, address: &ProtocolAddress) -> bool;
    async fn get_all_messages(&self, address: &ProtocolAddress) -> Vec<ServerEnvelope>;
    async fn lock_queue_for_persistence(&self, address: &ProtocolAddress);
    async fn unlock_queue_for_persistence(&self, address: &ProtocolAddress);
    async fn get_persisted_messages(
        &self,
        address: &ProtocolAddress,
        limit: i32,
    ) -> Vec<ServerEnvelope>;
    fn add_message_availability_listener(&mut self);
    fn remove_message_availability_listener(&mut self);
    async fn get_too_old_message_queues(&self, max_time: u64, limit: u8) -> Vec<String>;
    fn get_message_queue_key(&self, address: &ProtocolAddress) -> String;
    fn get_persist_in_progress_key(&self, address: &ProtocolAddress) -> String;
    fn get_message_queue_metadata_key(&self, address: &ProtocolAddress) -> String;
    fn get_queue_index_key(&self) -> String;
}
