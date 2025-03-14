use rustls_pemfile::{certs, private_key};
use sam_server::{start_server, state::ServerState, ServerConfig};
use std::env;
use std::fs::File;
use std::io::BufReader;
use std::sync::Arc;

#[tokio::main]
pub async fn main() {
    let _ = rustls::crypto::ring::default_provider().install_default();
    env_logger::init();
    let args: Vec<String> = env::args().collect();
    let state = ServerState::in_memory("test".to_string(), 600, 10);

    let cert_file = File::open(args[1].to_owned()).expect("Failed to open cert file");
    let mut cert_reader = BufReader::new(cert_file);
    let cert_chain = certs(&mut cert_reader)
        .map(|cert| cert.expect("Certificate should be there"))
        .collect::<Vec<_>>();

    let key_file = File::open(args[2].to_owned()).expect("Failed to open key file");
    let mut key_reader = BufReader::new(key_file);
    let key = private_key(&mut key_reader)
        .expect("Should find key")
        .expect("Key should be there");

    let server_config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(cert_chain, key)
        .expect("Failed to create rustls::ServerConfig");

    let config = ServerConfig {
        state,
        addr: "127.0.0.1:8080"
            .parse()
            .expect("Unable to parse socket address"),
        tls_config: Some(Arc::new(server_config)),
    };
    start_server(config).await.unwrap();
}
