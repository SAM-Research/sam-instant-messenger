use crate::ServerError;
use derive_more::{Display, Error, From};

#[derive(Debug, Display, Error, From)]
pub enum WebSocketError {
    WebSocketDecodeError,
    WebSocketDisconnected,
    WebSocketSendError,
}

#[derive(Debug, Display, Error, From)]
pub enum WebSocketSessionError {
    MessageHandling(ServerError),
    WebSocket(WebSocketError),
}
