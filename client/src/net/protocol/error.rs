use derive_more::Error;
use sam_common::sam_message::DeviceList;

use super::websocket::WebSocketError;

#[derive(Debug, Error)]
pub enum ProtocolError {
    EmptyMessage,
    MalformedServerMessage,
    InvalidCredentials,
    WebSocketError(WebSocketError),
    ExtraDevices(#[error(not(source))] Vec<DeviceList>),
    MissingDevices(#[error(not(source))] Vec<DeviceList>),
}

impl std::fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProtocolError::EmptyMessage => {
                write!(f, "ProtocolError::EmptyMessage")
            }
            ProtocolError::MalformedServerMessage => {
                write!(f, "ProtocolError::MalformedServerMessage")
            }
            ProtocolError::InvalidCredentials => {
                write!(f, "ProtocolError::InvalidCredentials")
            }
            ProtocolError::ExtraDevices(devices) => {
                write!(f, "ProtocolError::ExtraDevices({:?})", devices)
            }
            ProtocolError::MissingDevices(devices) => {
                write!(f, "ProtocolError::MissingDevices({:?})", devices)
            }
            ProtocolError::WebSocketError(err) => {
                write!(f, "ProtocolError::WebSocket({:?})", err)
            }
        }
    }
}
