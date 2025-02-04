use derive_more::derive::{Display, Error, From};
use libsignal_protocol::SignalProtocolError;
use sam_common::LibError;

pub type Result<T> = std::result::Result<T, ClientError>;

#[derive(Debug, Display, Error, From)]
pub enum ClientError {
    #[error(ignore)]
    Custom(String),
    SignalProtocol(SignalProtocolError),
    Lib(LibError),
}
