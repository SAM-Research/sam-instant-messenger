use clap::{Arg, Command};
use log::info;

use sam_server::{create_tls_config, start_server, state::ServerState, ServerConfig};
use std::sync::Arc;

#[tokio::main]
pub async fn main() {
    env_logger::init();
    let matches = Command::new("sam_server")
        .arg(
            Arg::new("cert")
                .short('c')
                .long("tls-certificate")
                .required(false)
                .help(".crt file (Server)")
                .requires("key"),
        )
        .arg(
            Arg::new("key")
                .short('k')
                .long("tls-key")
                .required(false)
                .help(".key file (Server)")
                .requires("cert"),
        )
        .arg(
            Arg::new("client_auth")
                .short('a')
                .long("authenticate-client")
                .required(false)
                .help(".crt file (Certificate Authority)")
                .requires("cert")
                .requires("key"),
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

    let tls = if let (Some(cert), Some(key), ca_cert) = (
        matches.get_one::<String>("cert"),
        matches.get_one::<String>("key"),
        matches.get_one::<String>("client_auth"),
    ) {
        info!("Using {}TLS", (if ca_cert.is_some() { "m" } else { "" }));
        info!("Cerificate: '{}'", cert);
        info!("Key: '{}'", key);
        let _ = rustls::crypto::ring::default_provider().install_default();

        Some(Arc::new(
            create_tls_config(cert, key, ca_cert.map(|x| x.as_str()))
                .expect("Can build TLS Config"),
        ))
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
