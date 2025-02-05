pub trait MessageCache {
    fn insert(&self, address: &ProtocolAddress, envelope: &mut Envelope, message_guid: &str);

    fn remove();

    fn has_messages() -> bool;

    fn get_all_messages();

    fn lock_queue_for_persistence();

    fn unlock_queue_for_persistence();

    fn get_persisted_messages();
    fn add_message_availability_listener();
    fn remove_message_availability_listener();
    fn get_too_old_message_queues();
}
