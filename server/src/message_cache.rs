use crate::redis_cache::MessageAvailabilityListener;
use libsignal_protocol::ProtocolAddress;
use sam_common::sam_message::ServerEnvelope;
use std::sync::{Arc, Mutex};

pub trait MessageCache<T: MessageAvailabilityListener> {
    fn insert(&self, address: &ProtocolAddress, envelope: &ServerEnvelope, message_guid: &str);
    fn remove(&self, address: &ProtocolAddress, message_guids: Vec<String>);
    fn has_messages(&self, address: &ProtocolAddress) -> bool;
    fn get_all_messages(&self, address: &ProtocolAddress);
    fn lock_queue_for_persistence(&self, address: &ProtocolAddress);
    fn unlock_queue_for_persistence(&self, address: &ProtocolAddress);
    fn get_persisted_messages(&self, address: &ProtocolAddress, limit: i32);
    fn add_message_availability_listener(
        &mut self,
        address: &ProtocolAddress,
        listener: Arc<Mutex<T>>,
    );
    fn remove_message_availability_listener(&mut self, address: &ProtocolAddress);
    fn get_too_old_message_queues(&self, max_time: u64, limit: u8);
    fn get_message_queue_key(&self, address: &ProtocolAddress);
    fn get_persist_in_progress_key(&self, address: &ProtocolAddress);
    fn get_message_queue_metadata_key(&self, address: &ProtocolAddress);
    fn get_queue_index_key(&self);
}
