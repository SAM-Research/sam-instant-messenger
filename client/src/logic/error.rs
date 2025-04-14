use derive_more::derive::{Display, Error, From};
use libsignal_core::curve::CurveError;
use libsignal_protocol::SignalProtocolError;

use crate::{encryption::error::EncryptionError, net::ApiClientError, storage::error::StoreError};

#[derive(Debug, Display, Error, From)]
pub enum LogicError {
    MissingDevices,
    FailedToProcessPrekeyBundle,
    Store(StoreError),
    Api(ApiClientError),
    Curve(CurveError),
    SignalProtocol(SignalProtocolError),
    Encryption(EncryptionError),
}
