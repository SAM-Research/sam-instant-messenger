use std::{io::BufReader, net::SocketAddr};

use serde::{Deserialize, Serialize};

use crate::{
    create_tls_config,
    error::{TlsConfigError, TlsError},
    ServerState, StateType,
};

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
    pub fn load(path: String) -> Result<Self, TlsConfigError> {
        let file = std::fs::File::open(path)?;
        let reader = BufReader::new(file);
        Ok(serde_json::from_reader(reader)?)
    }

    pub fn new(ca_cert_path: Option<String>, cert_path: String, key_path: String) -> Self {
        Self {
            ca_cert_path,
            cert_path,
            key_path,
        }
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
