mod utils;

use std::time::Duration;

use sam_client::{
    net::{
        http_client::HttpClientConfig,
        protocol::{client::ProtocolClient, WebSocketProtocolClientConfig},
        HttpClient,
    },
    storage::sqlite::{SqliteStoreConfig, SqliteStoreType},
    Client,
};
use tokio::time::sleep;

use crate::utils::server::TestServer;

async fn client(
    address: &str,
    username: &str,
    device_name: &str,
) -> Client<SqliteStoreType, HttpClient, ProtocolClient> {
    Client::from_registration()
        .username(username)
        .device_name(device_name)
        .store_config(SqliteStoreConfig::in_memory().await)
        .api_client_config(HttpClientConfig::new(address.to_string()))
        .protocol_config(WebSocketProtocolClientConfig::new(address.to_string()))
        .call()
        .await
        .expect("Can register Client")
}

/*#[tokio::test]
async fn test_websocket_alice_send_to_bob() {
    let _ = env_logger::try_init();
    let address = "127.0.0.1:9387";
    let mut server = TestServer::start(address).await;

    let mut alice = client(address, "alice", "alice device").await;
    let mut bob = client(address, "bob", "bob device").await;

    let bob_id = bob.account_id().await.expect("Bob can get his id");
    let mut bob_recv = bob.subscribe().await;

    alice
        .send_message(bob_id, "Hello bob!")
        .await
        .expect("Alice can send message");

    //let msg = bob_recv.recv().await.expect("Can decrypt");

    assert!(true)
}*/
