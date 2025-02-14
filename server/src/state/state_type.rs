use axum::extract::ws::Message;
use futures_util::{Sink, Stream};

use crate::managers::traits::{
    account_manager::AccountManager,
    device_manager::DeviceManager,
    key_manager::{LastResortKeyManager, PqPreKeyManager, PreKeyManager, SignedPreKeyManager},
    message_manager::MessageManager,
};

pub trait StateType: 'static {
    type AccountManager: AccountManager;
    type DeviceManager: DeviceManager;
    type MessageManager: MessageManager;
    type KeyManager: PreKeyManager + SignedPreKeyManager + PqPreKeyManager + LastResortKeyManager;
    type Socket: Stream<Item = Message> + Sink<Message> + Send + 'static;
}
