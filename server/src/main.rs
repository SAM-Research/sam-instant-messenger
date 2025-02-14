use sam_server::{
    managers::in_memory::{
        account::InMemoryAccountManager, device::InMemoryDeviceManager, keys::InMemoryKeyManager,
        message::InMemoryMessageManager,
    },
    start_server,
    state::{state_type::StateType, ServerState},
    ServerConfig,
};

struct InMemStateType;

impl StateType for InMemStateType {
    type AccountManager = InMemoryAccountManager;
    type DeviceManager = InMemoryDeviceManager;
    type MessageManager = InMemoryMessageManager;
    type KeyManager = InMemoryKeyManager;
}

#[tokio::main]
pub async fn main() {
    let state: ServerState<InMemStateType> = ServerState::new(
        InMemoryAccountManager::new(),
        InMemoryDeviceManager::new("secret".to_string()),
        InMemoryMessageManager::new(),
        InMemoryKeyManager::new(),
    );

    let config = ServerConfig {
        state: state,
        addr: "127.0.0.1:8080"
            .parse()
            .expect("Unable to parse socket address"),
        tls: None,
    };
    start_server(config).await.unwrap();
    println!("Hello World");
}
