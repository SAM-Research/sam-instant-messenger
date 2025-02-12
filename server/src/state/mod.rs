pub mod entities;
pub mod traits;

use std::sync::Arc;

use tokio::sync::Mutex;

use traits::state_type::StateType;

type AMutex<T> = Arc<Mutex<T>>;

pub struct ServerState<T: StateType> {
    pub accounts: AMutex<T::AccountManager>,
    pub devices: AMutex<T::DeviceManager>,
    pub messages: AMutex<T::MessageManager>,
    pub keys: AMutex<T::KeyManager>,
}

impl<T: StateType> Clone for ServerState<T> {
    fn clone(&self) -> Self {
        Self {
            accounts: self.accounts.clone(),
            devices: self.devices.clone(),
            messages: self.messages.clone(),
            keys: self.keys.clone(),
        }
    }
}

impl<T: StateType> ServerState<T> {
    pub async fn init(&mut self) {}
    pub async fn cleanup(&mut self) {}
}
