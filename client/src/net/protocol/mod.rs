use async_trait::async_trait;
use base64::{prelude::BASE64_STANDARD, Engine};
use error::ProtocolError;
use log::debug;
use rustls::ClientConfig;
use sam_common::{AccountId, DeviceId};
use std::sync::Arc;
use tokio_tungstenite::tungstenite::http;
use tokio_tungstenite::Connector;
use websocket::WebSocketClientConfig;

pub mod client;
pub mod decode;
pub mod error;
pub mod traits;
pub mod websocket;

pub use client::ProtocolClient;
pub use decode::{DeviceList, MessageStatus};
pub use traits::{ProtocolConfig, SamProtocolClient};

pub struct WebSocketProtocolClientConfig {
    base_url: String,
    config: Option<ClientConfig>,
    channel_buffer_size: usize,
}

impl WebSocketProtocolClientConfig {
    pub fn new(base_url: String, channel_buffer_size: usize) -> Self {
        Self {
            base_url,
            config: None,
            channel_buffer_size,
        }
    }

    pub fn new_with_tls(
        base_url: String,
        config: ClientConfig,
        channel_buffer_size: usize,
    ) -> Self {
        Self {
            base_url,
            config: Some(config),
            channel_buffer_size,
        }
    }
}

pub fn get_ws_url_and_connector(
    config: Option<ClientConfig>,
    base_url: String,
) -> (String, Option<Connector>) {
    match config {
        None => (format!("ws://{}", base_url), None),
        Some(config) => (
            format!("wss://{}", base_url),
            Some(Connector::Rustls(Arc::new(config))),
        ),
    }
}

pub fn get_ws_auth(account_id: AccountId, device_id: DeviceId, password: String) -> String {
    let basic = format!("{account_id}.{device_id}:{password}");
    format!("Basic {}", BASE64_STANDARD.encode(basic))
}

#[async_trait]
impl ProtocolConfig for WebSocketProtocolClientConfig {
    type ProtocolClient = ProtocolClient;

    async fn create(
        self,
        account_id: AccountId,
        device_id: DeviceId,
        password: String,
    ) -> Result<Self::ProtocolClient, ProtocolError> {
        let (url, connector) = get_ws_url_and_connector(self.config, self.base_url);
        let basic = get_ws_auth(account_id, device_id, password);
        let ws_client = WebSocketClientConfig::builder()
            .maybe_tls(connector)
            .url(format!("{}/api/v1/websocket", url))
            .headers(vec![(
                http::header::AUTHORIZATION,
                http::HeaderValue::from_str(&basic)
                    .inspect_err(|e| debug!("{e}"))
                    .map_err(|_| ProtocolError::InvalidCredentials)?,
            )])
            .build()
            .into();
        Ok(ProtocolClient::new(ws_client, self.channel_buffer_size))
    }
}
