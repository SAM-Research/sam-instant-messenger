use crate::StateType;

mod account;
mod device;
mod keys;
mod message;

#[cfg(test)]
pub(super) mod test_utils;

use account::PostgresAccountManager;
use device::PostgresDeviceManager;
use keys::PostgresKeyManager;
use message::PostgresMessageManager;

#[derive(Clone)]
pub struct PostgresStateType;

impl StateType for PostgresStateType {
    type AccountManager = PostgresAccountManager;
    type DeviceManager = PostgresDeviceManager;
    type MessageManager = PostgresMessageManager;
    type KeyManagerType = PostgresKeyManager;
}
