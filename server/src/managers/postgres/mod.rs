use account::PostgresAccountManager;
use device::PostgresDeviceManager;
use keys::PostgresKeyManager;
use message::PostgresMessageManager;

use crate::StateType;

pub mod account;
pub mod device;
pub mod keys;
pub mod message;

#[derive(Clone)]
pub struct InMemStateType;

impl StateType for InMemStateType {
    type AccountManager = PostgresAccountManager;
    type DeviceManager = PostgresDeviceManager;
    type MessageManager = PostgresMessageManager;
    type KeyManager = PostgresKeyManager;
}
