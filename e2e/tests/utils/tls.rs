use rustls::pki_types::CertificateDer;
use rustls::{ClientConfig, RootCertStore, ServerConfig};
use rustls_pemfile::{certs, private_key};
use sam_client::net::protocol::error::ProtocolError;
use std::fs::File;
use std::io::BufReader;
use std::sync::Arc;

#[allow(unused)]
pub fn make_rustls_client_config(cert_path: &str) -> Result<ClientConfig, ProtocolError> {
    let ca_file = File::open(cert_path)
        .map_err(|e| e.to_string())
        .expect("Can read file");
    let mut reader = BufReader::new(ca_file);

    let certs: Vec<CertificateDer<'static>> = certs(&mut reader)
        .collect::<Result<_, _>>()
        .expect("Can extract certificate");

    let mut root_store = RootCertStore::empty();
    for cert in certs {
        root_store.add(cert).expect("Certificate is valid");
    }

    let config = ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();

    Ok(config)
}

#[allow(unused)]
pub fn make_rustls_server_config(cert_path: &str, key_path: &str) -> Arc<ServerConfig> {
    let cert_file = File::open(cert_path).expect("Failed to open cert file");
    let mut cert_reader = BufReader::new(cert_file);
    let cert_chain = certs(&mut cert_reader)
        .map(|cert| cert.expect("Certificate should be there"))
        .collect::<Vec<_>>();

    let key_file = File::open(key_path).expect("Failed to open key file");
    let mut key_reader = BufReader::new(key_file);
    let key = private_key(&mut key_reader)
        .expect("Should find key")
        .expect("Key should be there");

    let config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(cert_chain, key)
        .expect("Failed to create rustls::ServerConfig");

    Arc::new(config)
}
