use derive_more::derive::{Display, Error, From};
use libsignal_core::curve::CurveError;
use libsignal_protocol::SignalProtocolError;

use crate::{
    encryption::error::EncryptionError,
    net::ApiClientError,
    storage::error::{
        AccountStoreError, ContactStoreError, KeyStoreError, MessageStoreError, StoreCreationError,
    },
};

#[derive(Debug, Display, Error, From)]
pub enum LogicError {
    MissingDevices,
    FailedToProcessPrekeyBundle,
    Store(StoreCreationError),
    Api(ApiClientError),
    Curve(CurveError),
    SignalProtocol(SignalProtocolError),
    Encryption(EncryptionError),
    AccountStore(AccountStoreError),
    ContactStore(ContactStoreError),
    MessageStore(MessageStoreError),
    KeyStore(KeyStoreError),
}
