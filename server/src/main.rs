use std::io::BufReader;

use clap::{Arg, Command};
use log::{debug, error, info};

use sam_server::{
    config::ServerCliConfig, error::CliError, start_server, state::ServerState, ServerConfig,
};

const DEFAULT_ADDR: &str = "127.0.0.1:8080";
const DEFAULT_MESSAGE_BUFFER_SIZE: usize = 10;

fn welcome(config: &ServerCliConfig) {
    let addr = config.address.clone().unwrap_or(DEFAULT_ADDR.to_string());
    let buffer_size = config
        .message_buffer_size
        .unwrap_or(DEFAULT_MESSAGE_BUFFER_SIZE);
    info!("*********Configuration*********");
    info!("Server Address: {addr}");
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
            Arg::new("database_url")
                .short('d')
                .long("database-url")
                .required(true)
                .help("PostgreSQL connection url")
                .conflicts_with("config"),
        )
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
        let url = matches
            .get_one::<String>("database_url")
            .ok_or(CliError::ArgumentError("Expected Database url".to_string()))?;
        let addr = matches.get_one::<String>("server_address");
        let buffer_size = matches.get_one::<String>("buffer_size").ok_or(CliError::ArgumentError("Expected buffer size".to_string()))?
        .parse()
        .map_err(|_| {
            CliError::ArgumentError("Expected usize for deniable ratio. On 32 bit target, this is 4 bytes and on a 64 bit target, this is 8 bytes".to_string())
        })?;
        ServerCliConfig::new(url.clone(), addr.cloned(), Some(buffer_size), None, None)
    };

    if let Some(filter) = &config.logging {
        env_logger::builder().parse_filters(filter).init();
    } else {
        env_logger::init();
    }

    welcome(&config);

    let tls = if let Some(config) = config.tls {
        let _ = rustls::crypto::ring::default_provider().install_default();
        Some(config.try_into()?)
    } else {
        None
    };

    let state = match ServerState::connect(&config.database_url, DEFAULT_MESSAGE_BUFFER_SIZE).await
    {
        Ok(state) => state,
        Err(e) => Err(CliError::DatabaseError(e))?,
    };
    info!("Database: OK");

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

#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

#[tokio::main]
pub async fn main() {
    let res = cli().await;
    let _ = env_logger::try_init();
    match res {
        Ok(_) => info!("Goodbye!"),
        Err(e) => error!("Fatal Server Error: {}", e),
    }
}
