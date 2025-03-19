use clap::{Arg, Command};
use log::info;
use rustls_pemfile::{certs, private_key};
use sam_server::{start_server, state::ServerState, ServerConfig};
use std::fs::File;
use std::io::BufReader;
use std::sync::Arc;

#[tokio::main]
pub async fn main() {
    env_logger::init();
    let matches = Command::new("sam_server")
        .arg(
            Arg::new("cert")
                .short('c')
                .long("certificate")
                .required(false)
                .help(".crt file")
                .requires("key"),
        )
        .arg(
            Arg::new("key")
                .short('k')
                .long("key")
                .required(false)
                .help(".key file")
                .requires("cert"),
        )
        .arg(
            Arg::new("ip")
                .short('i')
                .long("ip")
                .required(false)
                .help("IP to run server on")
                .default_value("127.0.0.1"),
        )
        .arg(
            Arg::new("port")
                .short('p')
                .long("port")
                .required(false)
                .help("Port to run server on")
                .default_value("8080"),
        )
        .get_matches();

    let tls = if let (Some(cert), Some(key)) = (
        matches.get_one::<String>("cert"),
        matches.get_one::<String>("key"),
    ) {
        info!("Using TLS");
        info!("Cerificate: '{}'", cert);
        info!("Key: '{}'", key);
        let _ = rustls::crypto::ring::default_provider().install_default();
        let cert_file = File::open(cert).expect("Failed to open cert file");
        let mut cert_reader = BufReader::new(cert_file);
        let cert_chain = certs(&mut cert_reader)
            .map(|cert| cert.expect("Certificate should be there"))
            .collect::<Vec<_>>();

        let key_file = File::open(key).expect("Failed to open key file");
        let mut key_reader = BufReader::new(key_file);
        let key = private_key(&mut key_reader)
            .expect("Should find key")
            .expect("Key should be there");

        let server_config = rustls::ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(cert_chain, key)
            .expect("Failed to create rustls::ServerConfig");
        Some(Arc::new(server_config))
    } else {
        None
    };

    let ip = matches.get_one::<String>("ip").expect("IP has default");
    let port = matches.get_one::<String>("port").expect("Port has default");

    let addr = format!("{}:{}", ip, port);

    let state = ServerState::in_memory("test".to_string(), 600, 10);

    let config = ServerConfig {
        state,
        addr: addr.parse().expect("Unable to parse socket address"),
        tls_config: tls,
    };
    start_server(config).await.unwrap();
}
