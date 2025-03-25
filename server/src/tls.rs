use std::sync::Arc;

use crate::error::TlsError;
use rustls::server::WebPkiClientVerifier;
use rustls::RootCertStore;
use sam_common::tls::{load_certs, load_private_key};

pub fn create_tls_config(
    server_cert_path: &str,
    server_key_path: &str,
    ca_cert_path: Option<&str>,
) -> Result<rustls::ServerConfig, TlsError> {
    let certs = load_certs(server_cert_path)?;
    let keys = load_private_key(server_key_path)?.ok_or(TlsError::PrivateKeyWasNone)?;
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
