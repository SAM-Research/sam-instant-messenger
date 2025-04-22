use sam_server::{
    managers::{
        in_memory::{
            account::InMemoryAccountManager,
            device::InMemoryDeviceManager,
            keys::{
                InMemoryEcPreKeyManager, InMemoryLastResortPqPreKeyManager,
                InMemoryPqPreKeyManager, InMemorySignedPreKeyManager,
            },
            message::InMemoryMessageManager,
            InMemStateType,
        },
        postgres::{
            PostgresAccountManager, PostgresConnector, PostgresDeviceManager, PostgresStateType,
        },
        KeyManager,
    },
    start_server, ServerConfig, ServerState, StateType,
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
    pub async fn start<T: StateType>(
        address: &str,
        tls_config: Option<rustls::ServerConfig>,
        server_state: ServerState<T>,
    ) -> Self {
        let config = ServerConfig {
            state: server_state,
            addr: address.parse().expect("Unable to parse socket address"),
            tls_config,
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

pub async fn in_memory_server_state() -> ServerState<InMemStateType> {
    ServerState::new(
        InMemoryAccountManager::default(),
        InMemoryDeviceManager::new("test".to_string(), 600),
        InMemoryMessageManager::default(),
        KeyManager::new(
            InMemoryEcPreKeyManager::default(),
            InMemoryPqPreKeyManager::default(),
            InMemorySignedPreKeyManager::default(),
            InMemoryLastResortPqPreKeyManager::default(),
        ),
    )
}

pub async fn postgres_server_state() -> ServerState<PostgresStateType> {
    let connection_str = "postgres://test:test@127.0.0.1:5432/sam_test_db";
    let pool = PostgresConnector::connect(connection_str)
        .await
        .expect("can connect to the postgres test server")
        .pool();

    ServerState::<PostgresStateType>::new(
        PostgresAccountManager::new(pool.clone()),
        PostgresDeviceManager::create(pool, "TEST_LINK_SECRET", 30)
            .await
            .expect("Can save device manager configuration"),
        InMemoryMessageManager::default(),
        KeyManager {
            pre_keys: InMemoryEcPreKeyManager::default(),
            pq_pre_keys: InMemoryPqPreKeyManager::default(),
            signed_pre_keys: InMemorySignedPreKeyManager::default(),
            last_resort_keys: InMemoryLastResortPqPreKeyManager::default(),
        },
    )
}
