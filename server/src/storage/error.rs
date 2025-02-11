use derive_more::{Display, Error, From};
use sam_common::address::{AccountId, DeviceAddress, DeviceId};
use std::error::Error as StdError;

type BoxDynError = Box<dyn StdError + 'static + Send + Sync>;

#[derive(Debug, Display, Error)]
pub enum AccountStoreError {
    #[error(ignore)]
    AccountIdTaken(String),
    Database(BoxDynError),
    AccountNotFound(#[error(not(source))] AccountId),
}

#[derive(Debug, Display, Error)]
pub enum DeviceStoreError {
    AccountNotFound(#[error(not(source))] AccountId),
    #[error(ignore)]
    DeviceIdTaken(DeviceId),
    #[error(ignore)]
    DeviceNotFound(DeviceId),
    Database(BoxDynError),
}

#[derive(Debug, Display, From, Error)]
pub enum KeyStoreError {
    Database(BoxDynError),
    AddressNotFound(#[error(not(source))] DeviceAddress),
}

#[derive(Debug, Display, From, Error)]
pub enum MessageStoreError {
    Database(BoxDynError),
    NoMessagesInQueue,
}
