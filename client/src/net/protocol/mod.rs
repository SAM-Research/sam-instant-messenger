use base64::{prelude::BASE64_STANDARD, Engine};
use client::ProtocolClient;
use error::ProtocolError;
use sam_common::{AccountId, DeviceId};
use tokio_tungstenite::tungstenite::http;
use traits::ProtocolConfig;
use websocket::WebSocketClientConfig;

pub mod client;
pub mod error;
pub mod traits;
mod websocket;

pub struct WebSocketProtocolClientConfig {
    url: String,
}

impl WebSocketProtocolClientConfig {
    pub fn new(url: String) -> Self {
        Self { url }
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
        let basic = format!("{account_id}.{device_id}:{password}");
        let basic = format!("Basic {}", BASE64_STANDARD.encode(basic));
        let ws_client = WebSocketClientConfig::builder()
            .url(format!("ws://{}", self.url))
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
