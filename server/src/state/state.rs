use std::sync::Arc;

use tokio::sync::Mutex;

use super::traits::{
    account_manager::AccountManager, key_manager::KeyManager, message_manager::MessageManager,
};

type BoxMutex<T> = Arc<Mutex<Box<T>>>;

#[derive(Clone, bon::Builder)]
pub struct ServerState {
    pub accounts: BoxMutex<dyn AccountManager>,
    pub messages: BoxMutex<dyn MessageManager>,
    pub keys: BoxMutex<dyn KeyManager>,
    pub link_secret: String,
}

impl ServerState {
    pub async fn init(&mut self) {}
    pub async fn cleanup(&mut self) {}
}
