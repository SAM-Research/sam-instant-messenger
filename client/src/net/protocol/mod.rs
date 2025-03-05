use client::ProtocolClient;
use traits::ProtocolConfig;
use websocket::WebSocketClientConfig;

pub mod client;
pub mod error;
pub mod traits;
mod websocket;

pub struct WebSocketProtocolClientConfig {
    url: String,
}

#[async_trait::async_trait(?Send)]
impl ProtocolConfig for WebSocketProtocolClientConfig {
    type ProtocolClient = ProtocolClient;

    async fn create(self) -> Result<Self::ProtocolClient, error::ProtocolError> {
        let ws_client = WebSocketClientConfig::builder()
            .url(format!("ws://{}", self.url))
            .build()
            .into();
        Ok(ProtocolClient::new(ws_client))
    }
}
