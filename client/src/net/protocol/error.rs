use derive_more::Error;

use super::websocket::WebSocketError;

#[derive(Debug, Error)]
pub enum ProtocolError {
    EmptyMessage,
    MalformedServerMessage,
    InvalidCredentials,
    WebSocketError(WebSocketError),
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
            ProtocolError::WebSocketError(err) => {
                write!(f, "ProtocolError::WebSocket({:?})", err)
            }
        }
    }
}
