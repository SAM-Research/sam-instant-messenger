use axum::extract::ws::Message;
use futures_util::{Sink, Stream};

use super::{
    account_manager::AccountManager, device_manager::DeviceManager, key_manager::KeyManager,
    message_manager::MessageManager,
};

pub trait StateType: 'static {
    type AccountManager: AccountManager;
    type DeviceManager: DeviceManager;
    type MessageManager: MessageManager;
    type KeyManager: KeyManager;
    type Socket: Stream<Item = Message> + Sink<Message> + Send + 'static;
}
