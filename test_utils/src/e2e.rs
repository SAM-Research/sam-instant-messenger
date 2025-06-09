use rand::rngs::OsRng;
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
            keys::{
                PostgresEcPreKeyManager, PostgresLastResortPqPreKeyManager,
                PostgresPqPreKeyManager, PostgresSignedPreKeyManager,
            },
            PostgresAccountManager, PostgresConnector, PostgresDeviceManager,
            PostgresMessageManager, PostgresStateType,
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
        OsRng,
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
        OsRng,
        PostgresAccountManager::new(pool.clone()),
        PostgresDeviceManager::create(pool.clone(), "TEST_LINK_SECRET", 30)
            .await
            .expect("Can save device manager configuration"),
        PostgresMessageManager::new(pool.clone(), 10),
        KeyManager {
            pre_keys: PostgresEcPreKeyManager::new(pool.clone()),
            pq_pre_keys: PostgresPqPreKeyManager::new(pool.clone()),
            signed_pre_keys: PostgresSignedPreKeyManager::new(pool.clone()),
            last_resort_keys: PostgresLastResortPqPreKeyManager::new(pool),
        },
    )
}
