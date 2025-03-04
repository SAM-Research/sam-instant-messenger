use derive_more::derive::{Display, Error, From};
use libsignal_core::curve::CurveError;
use libsignal_protocol::SignalProtocolError;
use sam_common::LibError;

#[derive(Debug, Display, Error, From)]
pub enum ClientError {
    #[display("Failed to parse an invalid ServiceId: {_0}")]
    #[error(ignore)]
    InvalidAccountId(String),
    SignalProtocol(SignalProtocolError),
    #[from(ignore)]
    Database(#[error(not(source))] String),
    Lib(LibError),
    Curve(CurveError),
    NoAccountId,
    NoPassword,
    NoUsername,
    NoSession,
}
