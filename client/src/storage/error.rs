use derive_more::derive::{Display, Error, From};
use libsignal_core::curve::CurveError;
use libsignal_protocol::SignalProtocolError;

#[derive(Debug, Display, Error, From)]
pub enum StoreError {
    #[display("Failed to parse an invalid AccountId: {_0}")]
    #[error(ignore)]
    InvalidAccountId(String),
    NoDeviceId,
    NoAccountId,
    NoPassword,
    NoUsername,
    SendError,
    #[from(ignore)]
    Database(#[error(not(source))] String),
    SignalProtocol(SignalProtocolError),
    Curve(CurveError),
}
