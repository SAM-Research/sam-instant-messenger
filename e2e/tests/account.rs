use crate::utils::server::TestServer;

use sam_client::{
    net::{
        http_client::HttpClientConfig,
        protocol::{client::ProtocolClient, WebSocketProtocolClientConfig},
        HttpClient,
    },
    storage::sqlite::{SqliteStoreConfig, SqliteStoreType},
    Client,
};

mod utils;

pub async fn register_alice(
    address: String,
) -> Client<SqliteStoreType, HttpClient, ProtocolClient> {
    Client::from_registration()
        .username("Alice")
        .device_name("Alice's Device")
        .store_config(SqliteStoreConfig::in_memory().await)
        .api_client_config(HttpClientConfig::new(address.clone()))
        .protocol_config(WebSocketProtocolClientConfig::new(
            address + "/api/v1/websocket",
        ))
        .call()
        .await
        .expect("Can register Alice")
}

#[tokio::test]
pub async fn can_delete_a_client() {
    let _ = env_logger::try_init();
    let address = "127.0.0.1:9480".to_owned();
    let mut server = TestServer::start(&address).await;

    server
        .started_rx()
        .await
        .expect("Should be able to start server");

    let client = register_alice(address).await;

    let result = client.delete_account().await;
    assert!(
        result.is_ok(),
        "Error deleting account: {:?}",
        result.unwrap_err().1
    )
}
