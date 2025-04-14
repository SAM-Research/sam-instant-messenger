use std::io::BufReader;

use clap::{Arg, Command};
use log::{debug, error, info};

use sam_server::{
    config::ServerCliConfig, error::CliError, start_server, state::ServerState, ServerConfig,
};

const DEFAULT_ADDR: &str = "127.0.0.1:8080";
const DEFAULT_LINK_SECRET: &str = "verysecret";
const DEFAULT_PROVISION_TIMEOUT_SECS: u64 = 600;
const DEFAULT_MESSAGE_BUFFER_SIZE: usize = 10;

fn welcome(config: &ServerCliConfig) {
    let addr = config.address.clone().unwrap_or(DEFAULT_ADDR.to_string());
    let prov_timeout = config
        .provision_timeout
        .unwrap_or(DEFAULT_PROVISION_TIMEOUT_SECS);
    let buffer_size = config
        .message_buffer_size
        .unwrap_or(DEFAULT_MESSAGE_BUFFER_SIZE);
    info!("*********Configuration*********");
    info!("Server Address: {addr}");
    info!("Provision Timeout: {prov_timeout} seconds");
    info!("Message Buffer Size: {buffer_size}");
    if let Some(tls) = &config.tls {
        if let Some(ca) = &tls.ca_cert_path {
            info!("Connection Security: mTLS");
            info!("Certificate Authority: {}", ca);
        } else {
            info!("Connection Security: TLS");
        }
        info!("Server Certificate: {}", tls.cert_path);
        info!("Server Key: {}", tls.key_path);
    } else {
        info!("Connection Security: Insecure")
    }
    info!("*******************************");
}

async fn cli() -> Result<(), CliError> {
    let matches = Command::new("sam_server")
        .arg(
            Arg::new("server_address")
                .short('s')
                .long("server-address")
                .required(false)
                .help("IP to run server on")
                .default_value(DEFAULT_ADDR)
                .conflicts_with("config"),
        )
        .arg(
            Arg::new("link_secret")
                .short('l')
                .long("link-secret")
                .required(false)
                .help("Link secret used to create link signature")
                .default_value(DEFAULT_LINK_SECRET)
                .conflicts_with("config"),
        )
        .arg(
            Arg::new("provision_timeout")
                .short('p')
                .long("provision-timeout")
                .required(false)
                .help("Provision timeout for linking new devices in seconds")
                .default_value(DEFAULT_PROVISION_TIMEOUT_SECS.to_string())
                .conflicts_with("config"),
        )
        .arg(
            Arg::new("buffer_size")
                .short('m')
                .long("message-buffer-size")
                .required(false)
                .help("How many messages can be in a buffer channel before blocking behaviour")
                .default_value(DEFAULT_MESSAGE_BUFFER_SIZE.to_string())
                .conflicts_with("config"),
        )
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .required(false)
                .help("JSON Config path"),
        )
        .get_matches();

    let config = if let Some(config_path) = matches.get_one::<String>("config") {
        let file = std::fs::File::open(config_path)?;
        let reader = BufReader::new(file);
        ServerCliConfig::load(reader)?
    } else {
        let addr = matches.get_one::<String>("server_address");
        let link_secret = matches.get_one::<String>("link_secret");
        let prov_timeout = matches
            .get_one::<String>("provision_timeout")
            .ok_or(CliError::ArgumentError(
                "Expected provision timeout".to_string(),
            ))?
            .parse()
            .map_err(|_| {
                CliError::ArgumentError("Expected u64 for provision timeout".to_string())
            })?;
        let buffer_size = matches.get_one::<String>("buffer_size").ok_or(CliError::ArgumentError("Expected buffer size".to_string()))?
        .parse()
        .map_err(|_| {
            CliError::ArgumentError("Expected usize for deniable ratio. On 32 bit target, this is 4 bytes and on a 64 bit target, this is 8 bytes".to_string())
        })?;
        ServerCliConfig::new(
            addr.cloned(),
            link_secret.cloned(),
            Some(prov_timeout),
            Some(buffer_size),
            None,
        )
    };

    welcome(&config);

    let tls = if let Some(config) = config.tls {
        let _ = rustls::crypto::ring::default_provider().install_default();
        Some(config.try_into()?)
    } else {
        None
    };

    let state = ServerState::in_memory(
        config
            .link_secret
            .unwrap_or(DEFAULT_LINK_SECRET.to_string()),
        config
            .provision_timeout
            .unwrap_or(DEFAULT_PROVISION_TIMEOUT_SECS),
        config
            .message_buffer_size
            .unwrap_or(DEFAULT_MESSAGE_BUFFER_SIZE),
    );

    let config = ServerConfig {
        state,
        addr: config
            .address
            .unwrap_or(DEFAULT_ADDR.to_string())
            .parse()
            .inspect_err(|e| debug!("{e}"))
            .map_err(|_| CliError::AddressParseError)?,
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
