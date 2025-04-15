use std::sync::Arc;

use rustls::{server::WebPkiClientVerifier, ClientConfig, RootCertStore};
use sam_common::tls::{load_certs, load_private_key};

use crate::error::ServerTlsError;

use super::error::ClientTlsError;

#[derive(Clone)]
pub struct MutualTlsConfig {
    key_path: String,
    cert_path: String,
}

impl MutualTlsConfig {
    pub fn new(key_path: String, cert_path: String) -> Self {
        Self {
            key_path,
            cert_path,
        }
    }
}

pub fn create_tls_client_config(
    ca_cert_path: &str,
    mutual_config: Option<MutualTlsConfig>,
) -> Result<ClientConfig, ClientTlsError> {
    let certs = load_certs(ca_cert_path)?;
    let mut root_store = RootCertStore::empty();
    root_store.add_parsable_certificates(certs);

    Ok(if let Some(config) = mutual_config {
        let cert_chain = load_certs(&config.cert_path)?;
        let key_der =
            load_private_key(&config.key_path)?.ok_or(ClientTlsError::PrivateKeyWasNone)?;
        ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_client_auth_cert(cert_chain, key_der)?
    } else {
        ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth()
    })
}

pub fn create_tls_server_config(
    server_cert_path: &str,
    server_key_path: &str,
    ca_cert_path: Option<&str>,
) -> Result<rustls::ServerConfig, ServerTlsError> {
    let certs = load_certs(server_cert_path)?;
    let keys = load_private_key(server_key_path)?.ok_or(ServerTlsError::PrivateKeyWasNone)?;
    Ok(if let Some(ca_path) = ca_cert_path {
        let ca_cert = load_certs(ca_path)?;
        let mut root_ca = RootCertStore::empty();
        root_ca.add_parsable_certificates(ca_cert);
        let client_auth = WebPkiClientVerifier::builder(Arc::new(root_ca)).build()?;
        rustls::ServerConfig::builder()
            .with_client_cert_verifier(client_auth)
            .with_single_cert(certs, keys)?
    } else {
        rustls::ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(certs, keys)?
    })
}
