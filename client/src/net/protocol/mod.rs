use crate::net::protocol::error::ProtocolError;
use client::ProtocolClient;
use rustls::ClientConfig;
use std::sync::Arc;
use tokio_tungstenite::Connector;
use traits::ProtocolConfig;
use websocket::WebSocketClientConfig;

pub mod client;
pub mod error;
pub mod traits;
mod websocket;

pub struct WebSocketProtocolClientConfig {
    base_url: String,
    maybe_config: Option<ClientConfig>,
}

impl WebSocketProtocolClientConfig {
    pub fn new(base_url: String, maybe_config: Option<ClientConfig>) -> Self {
        Self {
            base_url,
            maybe_config,
        }
    }
}

#[async_trait::async_trait(?Send)]
impl ProtocolConfig for WebSocketProtocolClientConfig {
    type ProtocolClient = ProtocolClient;

    async fn create(self) -> Result<Self::ProtocolClient, ProtocolError> {
        let (url, maybe_connector) = match self.maybe_config {
            None => (format!("ws://{}", self.base_url), None),
            Some(config) => (
                format!("wss://{}", self.base_url),
                Some(Connector::Rustls(Arc::new(config))),
            ),
        };
        let ws_client = WebSocketClientConfig::builder()
            .maybe_tls(maybe_connector)
            .url(url)
            .build()
            .into();

        Ok(ProtocolClient::new(ws_client))
    }
}
