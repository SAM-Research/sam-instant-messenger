use crate::managers::traits::{
    account_manager::AccountManager,
    device_manager::DeviceManager,
    key_manager::{LastResortKeyManager, PqPreKeyManager, PreKeyManager, SignedPreKeyManager},
    message_manager::MessageManager,
};

pub trait StateType: 'static + Clone {
    type AccountManager: AccountManager;
    type DeviceManager: DeviceManager;
    type MessageManager: MessageManager;
    type KeyManager: PreKeyManager + SignedPreKeyManager + PqPreKeyManager + LastResortKeyManager;
}
