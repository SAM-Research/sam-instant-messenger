use derive_more::From;

use super::websocket::WebSocketError;

#[derive(From, Debug)]
pub enum SamProtocolError {
    WebSocketError(WebSocketError),
    MalformedServerResponse,
}
