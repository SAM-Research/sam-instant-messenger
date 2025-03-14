use base64::{prelude::BASE64_STANDARD, Engine};
use client::ProtocolClient;
use error::ProtocolError;
use rustls::ClientConfig;
use sam_common::{AccountId, DeviceId};
use std::sync::Arc;
use tokio_tungstenite::tungstenite::http;
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

    async fn create(
        self,
        account_id: AccountId,
        device_id: DeviceId,
        password: String,
    ) -> Result<Self::ProtocolClient, error::ProtocolError> {
        let (url, maybe_connector) = match self.maybe_config {
            None => (format!("ws://{}", self.base_url), None),
            Some(config) => (
                format!("wss://{}", self.base_url),
                Some(Connector::Rustls(Arc::new(config))),
            ),
        };
        let basic = format!("{account_id}.{device_id}:{password}");
        let basic = format!("Basic {}", BASE64_STANDARD.encode(basic));
        let ws_client = WebSocketClientConfig::builder()
            .maybe_tls(maybe_connector)
            .url(format!("{}/api/v1/websocket", url))
            .headers(vec![(
                http::header::AUTHORIZATION,
                http::HeaderValue::from_str(&basic)
                    .map_err(|_| ProtocolError::InvalidCredentials)?,
            )])
            .build()
            .into();

        Ok(ProtocolClient::new(ws_client))
    }
}
