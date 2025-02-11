use super::{
    account_manager::AccountManager, device_manager::DeviceManager, key_manager::KeyManager,
    message_manager::MessageManager,
};

pub trait StateType: 'static {
    type AccountManager: AccountManager;
    type DeviceManager: DeviceManager;
    type MessageManager: MessageManager;
    type KeyManager: KeyManager;
}
