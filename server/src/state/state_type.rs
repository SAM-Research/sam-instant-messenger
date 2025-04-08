use crate::managers::{
    traits::{
        account_manager::AccountManager, device_manager::DeviceManager,
        message_manager::MessageManager,
    },
    KeyManagerType,
};

pub trait StateType: 'static + Clone {
    type AccountManager: AccountManager;
    type DeviceManager: DeviceManager;
    type MessageManager: MessageManager;
    type KeyManagerType: KeyManagerType;
}
