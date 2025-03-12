use crate::net::protocol::error::ProtocolError;
use client::ProtocolClient;
use rustls::pki_types::CertificateDer;
use rustls::{ClientConfig, RootCertStore};
use rustls_pemfile::certs;
use std::fs::File;
use std::io::BufReader;
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
    cert_path: Option<String>,
}

impl WebSocketProtocolClientConfig {
    pub fn new(base_url: String, cert_path: Option<String>) -> Self {
        Self {
            base_url,
            cert_path,
        }
    }

    fn make_rustls_client_config(cert_path: &str) -> Result<ClientConfig, ProtocolError> {
        let ca_file = File::open(cert_path)
            .map_err(|e| e.to_string())
            .map_err(|_| ProtocolError::WrongCertificatePath)?;
        let mut reader = BufReader::new(ca_file);

        let certs: Vec<CertificateDer<'static>> = certs(&mut reader)
            .collect::<Result<_, _>>()
            .map_err(|_| ProtocolError::FailedToExtractCertificate)?;

        let mut root_store = RootCertStore::empty();
        for cert in certs {
            root_store
                .add(cert)
                .map_err(|_| ProtocolError::InvalidCertificate)?;
        }

        let config = ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth();

        Ok(config)
    }
}

#[async_trait::async_trait(?Send)]
impl ProtocolConfig for WebSocketProtocolClientConfig {
    type ProtocolClient = ProtocolClient;

    async fn create(self) -> Result<Self::ProtocolClient, ProtocolError> {
        let (url, maybe_connector) = match self.cert_path {
            None => (format!("ws://{}", self.base_url), None),
            Some(path) => {
                let config = Self::make_rustls_client_config(path.as_str())?;
                (
                    format!("wss://{}", self.base_url),
                    Some(Connector::Rustls(Arc::new(config))),
                )
            }
        };
        let ws_client = WebSocketClientConfig::builder()
            .maybe_tls(maybe_connector)
            .url(url)
            .build()
            .into();

        Ok(ProtocolClient::new(ws_client))
    }
}
