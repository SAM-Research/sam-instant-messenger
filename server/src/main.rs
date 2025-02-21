use sam_server::{start_server, state::ServerState, ServerConfig};

#[tokio::main]
pub async fn main() {
    env_logger::init();
    let state = ServerState::in_memory("test".to_string(), 10);

    let config = ServerConfig {
        state,
        addr: "127.0.0.1:8080"
            .parse()
            .expect("Unable to parse socket address"),
        tls: None,
    };
    start_server(config).await.unwrap();
}
