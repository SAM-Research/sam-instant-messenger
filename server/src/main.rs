use std::io::BufReader;

use clap::{Arg, Command};
use log::{debug, error, info};

use sam_server::{
    config::TlsConfig, error::CLIError, start_server, state::ServerState, ServerConfig,
};

async fn cli() -> Result<(), CLIError> {
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
        .arg(
            Arg::new("config")
                .short('t')
                .long("tls-config")
                .required(false)
                .help("JSON TLS Config path")
                .conflicts_with("key")
                .conflicts_with("cert")
                .conflicts_with("client_auth"),
        )
        .get_matches();

    let tls_config = if let Some(config_path) = matches.get_one::<String>("config") {
        let file = std::fs::File::open(config_path)?;
        let reader = BufReader::new(file);
        Some(TlsConfig::load(reader)?)
    } else if let (Some(cert), Some(key), ca_cert) = (
        matches.get_one::<String>("cert"),
        matches.get_one::<String>("key"),
        matches.get_one::<String>("client_auth"),
    ) {
        Some(TlsConfig {
            ca_cert_path: ca_cert.map(|s| s.to_string()),
            cert_path: cert.to_string(),
            key_path: key.to_string(),
        })
    } else {
        None
    };

    let tls = if let Some(config) = tls_config {
        let is_mutual = config.ca_cert_path.is_some();
        info!("Using {}TLS", (if is_mutual { "m" } else { "" }));
        info!("Cerificate: '{}'", config.cert_path);
        info!("Key: '{}'", config.key_path);
        if let Some(path) = &config.ca_cert_path {
            info!("CA Cerificate: '{}'", path);
        }

        let _ = rustls::crypto::ring::default_provider().install_default();
        Some(config.try_into()?)
    } else {
        None
    };

    let ip = matches
        .get_one::<String>("ip")
        .ok_or(CLIError::ArgumentError("IP has default".to_string()))?;
    let port = matches
        .get_one::<String>("port")
        .ok_or(CLIError::ArgumentError("Port has default".to_string()))?;

    let addr = format!("{}:{}", ip, port);

    let state = ServerState::in_memory("test".to_string(), 600, 10);

    let config = ServerConfig {
        state,
        addr: addr
            .parse()
            .inspect_err(|e| debug!("{e}"))
            .map_err(|_| CLIError::AddressParseError)?,
        tls_config: tls,
    };
    Ok(start_server(config).await?)
}

#[tokio::main]
pub async fn main() {
    env_logger::init();
    match cli().await {
        Ok(_) => info!("Goodbye!"),
        Err(e) => error!("Fatal CLI Error: {}", e),
    };
}
