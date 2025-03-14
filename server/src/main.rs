use axum_server::tls_rustls::RustlsConfig;
use sam_server::{start_server, state::ServerState, ServerConfig};
use std::env;

#[tokio::main]
pub async fn main() {
    let _ = rustls::crypto::ring::default_provider().install_default();
    env_logger::init();
    let args: Vec<String> = env::args().collect();
    let state = ServerState::in_memory("test".to_string(), 600, 10);

    let rustls_config = RustlsConfig::from_pem_file(args[1].to_owned(), args[2].to_owned())
        .await
        .expect("Can create config from cert files");

    let config = ServerConfig {
        state,
        addr: "127.0.0.1:8080"
            .parse()
            .expect("Unable to parse socket address"),
        maybe_tls_config: Some(rustls_config),
    };
    start_server(config).await.unwrap();
}
