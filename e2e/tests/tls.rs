use crate::utils::server::TestServer;
use axum_server::tls_rustls::RustlsConfig;
use rustls::pki_types::CertificateDer;
use rustls::{ClientConfig, RootCertStore};
use rustls_pemfile::certs;
use sam_client::net::http_client::HttpClientConfig;
use sam_client::net::protocol::error::ProtocolError;
use sam_client::net::protocol::WebSocketProtocolClientConfig;
use sam_client::storage::sqlite::SqliteStoreConfig;
use sam_client::Client;
use std::fs::File;
use std::io::BufReader;

mod utils;

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

#[tokio::test]
pub async fn one_client_can_register_with_tls() {
    let _ = env_logger::try_init();
    let _ = rustls::crypto::ring::default_provider().install_default();
    let address = "127.0.0.1:9380".to_owned();
    let tls_config = RustlsConfig::from_pem_file("./cert/server.crt", "./cert/server.key")
        .await
        .expect("Can create config from crt files");
    let client_config = make_rustls_client_config("./cert/rootCA.crt").expect("Can make config");
    let mut server = TestServer::start("127.0.0.1:9380", Some(tls_config)).await;

    server
        .started_rx()
        .await
        .expect("Should be able to start server");

    let client = Client::from_registration()
        .username("Alice")
        .device_name("Alice's Device")
        .store_config(SqliteStoreConfig::in_memory().await)
        .api_client_config(HttpClientConfig::new(
            address.clone(),
            Some(client_config.clone()),
        ))
        .protocol_config(WebSocketProtocolClientConfig::new(
            address,
            Some(client_config.clone()),
        ))
        .call()
        .await;

    assert!(client.is_ok());
}
