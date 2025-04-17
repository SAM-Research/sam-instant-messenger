use crate::{ServerState, StateType};

mod account;
mod device;

#[cfg(test)]
pub(super) mod test_utils;

pub use account::PostgresAccountManager;
pub use device::PostgresDeviceManager;
use sqlx::postgres::PgPoolOptions;

use super::in_memory::{
    keys::{
        InMemoryEcPreKeyManager, InMemoryKeyManager, InMemoryLastResortPqPreKeyManager,
        InMemoryPqPreKeyManager, InMemorySignedPreKeyManager,
    },
    message::InMemoryMessageManager,
};

#[derive(Clone)]
pub struct PostgresStateType;

impl StateType for PostgresStateType {
    type AccountManager = PostgresAccountManager;
    type DeviceManager = PostgresDeviceManager;
    // TODO: Replace with postgres as they are implemented
    type MessageManager = InMemoryMessageManager;
    type KeyManagerType = InMemoryKeyManager;
}
