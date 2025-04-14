use derive_more::derive::{Display, Error, From};
use libsignal_core::curve::CurveError;
use libsignal_protocol::SignalProtocolError;
use sam_common::LibError;

use crate::net::{protocol::error::ProtocolError, ApiClientError};
use crate::storage::error::StoreError;

#[derive(Debug, Display, Error, From)]
pub enum ClientError {
    #[display("Failed to parse an invalid AccountId: {_0}")]
    #[error(ignore)]
    InvalidAccountId(String),
    SignalProtocol(SignalProtocolError),
    #[from(ignore)]
    Lib(LibError),
    Curve(CurveError),
    Api(ApiClientError),
    Protocol(ProtocolError),
    Store(StoreError),
    MissingDevices,
    FailedToProcessPrekeyBundle,
    FailedToUnpadMessage,
}
