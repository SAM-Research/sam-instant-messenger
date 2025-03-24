use rustls::{ClientConfig, RootCertStore};
use sam_common::tls::{load_certs, load_private_key};

use super::error::TLSError;

#[derive(Clone)]
pub struct MutualTLSConfig {
    key_path: String,
    cert_path: String,
}

impl MutualTLSConfig {
    pub fn new(key_path: String, cert_path: String) -> Self {
        Self {
            key_path,
            cert_path,
        }
    }
}

pub fn create_tls_config(
    servre_cert_path: &str,
    mutual_config: Option<MutualTLSConfig>,
) -> Result<ClientConfig, TLSError> {
    let certs = load_certs(servre_cert_path)?;
    let mut root_store = RootCertStore::empty();
    root_store.add_parsable_certificates(certs);

    Ok(if let Some(config) = mutual_config {
        let cert_chain = load_certs(&config.cert_path)?;
        let key_der = load_private_key(&config.key_path)?.ok_or(TLSError::PrivateKeyWasNone)?;
        ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_client_auth_cert(cert_chain, key_der)?
    } else {
        ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth()
    })
}
