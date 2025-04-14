use derive_more::derive::{Display, Error, From};
use libsignal_protocol::SignalProtocolError;

use crate::storage::error::{AccountStoreError, ContactStoreError, StoreCreationError};

#[derive(Debug, Display, Error, From)]
pub enum EncryptionError {
    FailedToUnpadMessage,
    Store(StoreCreationError),
    SignalProtocol(SignalProtocolError),
    #[display("Failed to parse an invalid AccountId: {_0}")]
    #[error(ignore)]
    InvalidAccountId(String),
    AccountStore(AccountStoreError),
    ContactStore(ContactStoreError),
}
