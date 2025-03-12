use sam_server::server::CertificatePaths;
use sam_server::{start_server, state::ServerState, ServerConfig};
use std::env;

#[tokio::main]
pub async fn main() {
    env_logger::init();
    let args: Vec<String> = env::args().collect();
    let cert_paths = CertificatePaths {
        key: args[1].to_owned(),
        cert: args[2].to_owned(),
    };
    let state = ServerState::in_memory("test".to_string(), 600, 10);

    let config = ServerConfig {
        state,
        addr: "127.0.0.1:8080"
            .parse()
            .expect("Unable to parse socket address"),
        tls: Some(cert_paths),
    };
    start_server(config).await.unwrap();
}
