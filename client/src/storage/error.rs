use derive_more::derive::{Display, Error, From};
use libsignal_core::curve::CurveError;
use libsignal_protocol::SignalProtocolError;

#[derive(Debug, Display, Error, From)]
pub enum StoreCreationError {
    Database(DatabaseError),
    SignalProtocol(SignalProtocolError),
    Curve(CurveError),
}

#[derive(Debug, Display, Error, From)]
pub enum AccountStoreError {
    Database(DatabaseError),
    NoAccountId,
    #[display("Failed to parse an invalid AccountId: {_0}")]
    InvalidAccountId(#[error(not(source))] String),
    NoPassword,
    NoUsername,
    NoDeviceId,
}

#[derive(Debug, Display, Error, From)]
pub enum ContactStoreError {
    Database(DatabaseError),
}

#[derive(Debug, Display, Error, From)]
pub enum MessageStoreError {
    Database(DatabaseError),
    SendError,
}

#[derive(Debug, Display, Error, From)]
pub enum KeyStoreError {
    Database(DatabaseError),
    SignalProtocol(SignalProtocolError),
}

#[derive(Debug, Display, Error, From)]
pub enum DatabaseError {
    Database(#[error(not(source))] String),
    FailedToConnect,
    FailedToMigrate,
}
