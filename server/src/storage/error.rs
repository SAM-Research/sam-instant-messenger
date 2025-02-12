use derive_more::{Display, Error, From};
use sam_common::address::{AccountId, DeviceAddress, DeviceId};
use std::error::Error as StdError;

type BoxDynError = Box<dyn StdError + 'static + Send + Sync>;

#[derive(Debug, Display, Error)]
pub enum AccountStoreError {
    AccountIdTaken(#[error(not(source))] AccountId),
    AccountNotFound(#[error(not(source))] AccountId),
    Database(BoxDynError),
}

#[derive(Debug, Display, Error)]
pub enum DeviceStoreError {
    AccountNotFound(#[error(not(source))] AccountId),
    DeviceIdTaken(#[error(not(source))] DeviceId),
    DeviceNotFound(#[error(not(source))] DeviceId),
    Database(BoxDynError),
}

#[derive(Debug, Display, From, Error)]
pub enum KeyStoreError {
    AddressNotFound(#[error(not(source))] DeviceAddress),
    Database(BoxDynError),
}

#[derive(Debug, Display, From, Error)]
pub enum MessageStoreError {
    Database(BoxDynError),
}

#[derive(Debug, Display, From, Error)]
pub enum DatabaseError {
    Account(AccountStoreError),
    Device(DeviceStoreError),
    Key(KeyStoreError),
    Message(MessageStoreError),
}
