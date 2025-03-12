mod utils;

use std::time::Duration;

use log::error;
use sam_client::{
    net::{
        http_client::HttpClientConfig,
        protocol::{client::ProtocolClient, traits::MessageStatus, WebSocketProtocolClientConfig},
        HttpClient,
    },
    storage::sqlite::{SqliteStoreConfig, SqliteStoreType},
    Client,
};
use tokio::time::timeout;

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

#[tokio::test]
async fn test_websocket_alice_send_to_bob() {
    let _ = env_logger::try_init();
    let address = "127.0.0.1:9387";
    let mut server = TestServer::start(address).await;

    let mut alice = client(address, "alice", "alice device").await;
    let mut bob = client(address, "bob", "bob device").await;

    let alice_id = alice.account_id().await.expect("alice can get her id");
    let bob_id = bob.account_id().await.expect("Bob can get his id");

    let mut bob_recv = bob.subscribe().await;
    error!("Alice: {}", alice_id);
    error!("Bob: {}", bob_id);
    alice
        .fetch_prekeys(bob_id, None)
        .await
        .expect("Can fetch bob keys");
    let status = alice
        .send_message(bob_id, "Hello bob!")
        .await
        .expect("Alice can send message");

    bob.process_messages()
        .await
        .expect("Bob can process messages");

    let res = timeout(Duration::from_millis(300), bob_recv.recv())
        .await
        .expect("Bob receives message in time")
        .expect("receiver works");

    assert!(matches!(status, MessageStatus::Ok))
}
