use derive_more::derive::{Display, Error, From};
use libsignal_protocol::SignalProtocolError;

use crate::storage::error::StoreError;

#[derive(Debug, Display, Error, From)]
pub enum EncryptionError {
    FailedToUnpadMessage,
    Store(StoreError),
    SignalProtocol(SignalProtocolError),
    #[display("Failed to parse an invalid AccountId: {_0}")]
    #[error(ignore)]
    InvalidAccountId(String),
}
