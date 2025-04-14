use derive_more::derive::{Display, Error, From};
use libsignal_protocol::SignalProtocolError;

use crate::encryption::error::EncryptionError;
use crate::logic::LogicError;
use crate::net::{protocol::error::ProtocolError, ApiClientError};
use crate::storage::error::{AccountStoreError, StoreCreationError};

#[derive(Debug, Display, Error, From)]
pub enum ClientError {
    SignalProtocol(SignalProtocolError),
    Api(ApiClientError),
    Protocol(ProtocolError),
    StoreCreation(StoreCreationError),
    AccountStore(AccountStoreError),
    Logic(LogicError),
    Encryption(EncryptionError),
}
