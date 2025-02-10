use derive_more::derive::{Display, Error, From};
use sam_common::LibError;

use crate::storage::error::{
    AccountStoreError, DeviceStoreError, KeyStoreError, MessageStoreError,
};

#[derive(Debug, Display, Error, From)]
pub enum ServerError {
    #[error(ignore)]
    Custom(String),
    Lib(LibError),
    AccountStore(AccountStoreError),
    DeviceStore(DeviceStoreError),
    KeyStore(KeyStoreError),
    Message(MessageStoreError),
}
