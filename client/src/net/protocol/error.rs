use derive_more::{Display, Error, From};
use sam_net::error::WebSocketError;

#[derive(Debug, Error, Display, From)]
pub enum ProtocolError {
    InvalidCredentials,
    ReceivedWrongResponseId,
    DecodeError(DecodeError),
    WebSocketError(WebSocketError),
}

#[derive(Debug, Error, Display, From)]
pub enum DecodeError {
    MalformedServerMessage,
    EmptyMessage,
}
