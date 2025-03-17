use derive_more::{Display, Error};

use super::websocket::WebSocketError;

#[derive(Debug, Error, Display)]
pub enum ProtocolError {
    MalformedServerMessage,
    InvalidCredentials,
    ReceivedWrongResponseId,
    EmptyMessage,
    WebSocketError(WebSocketError),
}
