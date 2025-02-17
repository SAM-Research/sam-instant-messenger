use sam_server::{start_server, state::ServerState, ServerConfig};

#[tokio::main]
pub async fn main() {
    let config = ServerConfig {
        state: ServerState::in_memory_default("test".to_string()),
        addr: "127.0.0.1:8080"
            .parse()
            .expect("Unable to parse socket address"),
        tls: None,
    };
    start_server(config).await.unwrap();
}
