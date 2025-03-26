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
pub struct TlsConfig {
    pub ca_cert_path: Option<String>,
    pub cert_path: String,
    pub key_path: String,
}

impl TlsConfig {
    pub fn load<R: std::io::Read>(reader: R) -> Result<Self, serde_json::Error> {
        serde_json::from_reader(reader)
    }
}

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
