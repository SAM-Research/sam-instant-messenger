pub mod state_type;
use state_type::StateType;

use crate::managers::KeyManager;

#[derive(Clone)]
pub struct ServerState<T: StateType> {
    pub accounts: T::AccountManager,
    pub devices: T::DeviceManager,
    pub messages: T::MessageManager,
    pub keys: KeyManager<T::KeyManagerType>,
}

impl<T: StateType> ServerState<T> {
    pub fn new(
        account: T::AccountManager,
        device: T::DeviceManager,
        message: T::MessageManager,
        key: KeyManager<T::KeyManagerType>,
    ) -> Self {
        Self {
            accounts: account,
            devices: device,
            messages: message,
            keys: key,
        }
    }
}
