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
        postgres::{PostgresAccountManager, PostgresDeviceManager, PostgresStateType},
        KeyManager,
    },
    start_server, ServerConfig, ServerState,
};

use sqlx::postgres::PgPoolOptions;
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
    pub async fn start(address: &str, tls_config: Option<rustls::ServerConfig>) -> Self {
        let config = ServerConfig {
            state: postgres_server_state().await,
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

pub fn in_memory_server_state() -> ServerState<InMemStateType> {
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
    let pool = PgPoolOptions::new()
        .connect(connection_str)
        .await
        .expect("Can connect to the database");

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
