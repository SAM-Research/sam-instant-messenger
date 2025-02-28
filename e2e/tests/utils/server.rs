use sam_server::{
    managers::in_memory::{
        account::InMemoryAccountManager, device::InMemoryDeviceManager, keys::InMemoryKeyManager,
        message::InMemoryMessageManager, InMemStateType,
    },
    start_server, ServerConfig, ServerState,
};

use tokio::{
    sync::oneshot::{self, Receiver},
    task::JoinHandle,
};

pub struct TestServer {
    thread: JoinHandle<Result<(), std::io::Error>>,
    started_rx: Receiver<()>,
}

impl Drop for TestServer {
    fn drop(&mut self) {
        self.thread.abort();
    }
}

impl TestServer {
    pub async fn start(address: &str) -> Self {
        let config = ServerConfig {
            state: in_memory_server_state(),
            addr: address.parse().expect("Unable to parse socket address"),
            tls: None,
        };
        let (tx, started_rx) = oneshot::channel::<()>();
        let thread = tokio::spawn(async move {
            let server = start_server(config);
            tx.send(())
                .expect("should be able to inform other thread that server is started");
            server.await
        });
        Self { thread, started_rx }
    }

    pub fn started_rx(&mut self) -> &mut Receiver<()> {
        &mut self.started_rx
    }
}

pub fn in_memory_server_state() -> ServerState<InMemStateType> {
    ServerState::new(
        InMemoryAccountManager::default(),
        InMemoryDeviceManager::new("test".to_string(), 600),
        InMemoryMessageManager::default(),
        InMemoryKeyManager::default(),
    )
}
