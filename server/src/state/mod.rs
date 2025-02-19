pub mod state_type;
use state_type::StateType;

pub struct ServerState<T: StateType> {
    pub accounts: T::AccountManager,
    pub devices: T::DeviceManager,
    pub messages: T::MessageManager,
    pub keys: T::KeyManager,
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
    pub fn new(
        account: T::AccountManager,
        device: T::DeviceManager,
        message: T::MessageManager,
        key: T::KeyManager,
    ) -> Self {
        Self {
            accounts: account,
            devices: device,
            messages: message,
            keys: key,
        }
    }
}
