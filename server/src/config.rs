use std::net::SocketAddr;

use serde::{Deserialize, Serialize};

use crate::{create_tls_config, error::TlsError, ServerState, StateType};

pub struct ServerConfig<T: StateType> {
    pub state: ServerState<T>,
    pub addr: SocketAddr,
    pub tls_config: Option<rustls::ServerConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerCliConfig {
    pub address: Option<String>,
    pub link_secret: Option<String>,
    pub provision_timeout: Option<u64>,
    pub message_buffer_size: Option<usize>,
    pub tls: Option<TlsConfig>,
}

impl ServerCliConfig {
    pub fn new(
        address: Option<String>,
        link_secret: Option<String>,
        provision_timeout: Option<u64>,
        message_buffer_size: Option<usize>,
        tls: Option<TlsConfig>,
    ) -> Self {
        Self {
            address,
            link_secret,
            provision_timeout,
            message_buffer_size,
            tls,
        }
    }

    pub fn load<R: std::io::Read>(reader: R) -> Result<Self, serde_json::Error> {
        serde_json::from_reader(reader)
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TlsConfig {
    pub ca_cert_path: Option<String>,
    pub cert_path: String,
    pub key_path: String,
}

impl TlsConfig {}

impl TryInto<rustls::ServerConfig> for TlsConfig {
    type Error = TlsError;

    fn try_into(self) -> Result<rustls::ServerConfig, Self::Error> {
        create_tls_config(
            &self.cert_path,
            &self.key_path,
            self.ca_cert_path.as_deref(),
        )
    }
}
