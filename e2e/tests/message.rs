mod utils;

use std::time::Duration;

use libsignal_protocol::IdentityKeyPair;
use sam_client::{
    net::{
        http_client::HttpClientConfig,
        protocol::{client::ProtocolClient, traits::MessageStatus, WebSocketProtocolClientConfig},
        HttpClient,
    },
    storage::inmem::{InMemoryStoreConfig, InMemoryStoreType},
    Client,
};
use sam_common::api::LinkDeviceToken;
use tokio::time::timeout;

use crate::utils::server::TestServer;

async fn client(
    address: &str,
    username: &str,
    device_name: &str,
) -> Client<InMemoryStoreType, HttpClient, ProtocolClient> {
    Client::from_registration()
        .username(username)
        .device_name(device_name)
        .store_config(InMemoryStoreConfig::default())
        .api_client_config(HttpClientConfig::new(address.to_string()))
        .protocol_config(WebSocketProtocolClientConfig::new(address.to_string()))
        .call()
        .await
        .expect("Can register Client")
}

async fn client_device(
    address: &str,
    device_name: &str,
    id_pair: IdentityKeyPair,
    token: LinkDeviceToken,
) -> Client<InMemoryStoreType, HttpClient, ProtocolClient> {
    Client::from_provisioning()
        .store_config(InMemoryStoreConfig::default())
        .api_client_config(HttpClientConfig::new(address.to_string()))
        .protocol_config(WebSocketProtocolClientConfig::new(address.to_string()))
        .device_name(device_name)
        .id_key_pair(id_pair)
        .token(token)
        .call()
        .await
        .expect("Can provision device")
}

/*
   PORTS USED: 9180-9189
*/

#[tokio::test]
async fn test_alice_send_to_bob() {
    let address = "127.0.0.1:9180";
    let mut server = TestServer::start(address).await;
    server
        .started_rx()
        .await
        .expect("Should be able to start server");

    let mut alice = client(address, "alice", "alice device").await;
    let mut bob = client(address, "bob", "bob device").await;

    let bob_id = bob.account_id().await.expect("Bob can get his id");

    let mut bob_recv = bob.subscribe();
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

    let msg = String::from_utf8_lossy(res.content_bytes());

    assert!(matches!(status, MessageStatus::Ok));
    assert!(msg == "Hello bob!")
}

#[tokio::test]
async fn test_alice_send_to_bob_offline() {
    let address = "127.0.0.1:9181";
    let mut server = TestServer::start(address).await;
    server
        .started_rx()
        .await
        .expect("Should be able to start server");

    let mut alice = client(address, "alice", "alice device").await;
    let mut bob = client(address, "bob", "bob device").await;

    let bob_id = bob.account_id().await.expect("Bob can get his id");

    bob.disconnect().await.expect("Bob can disconnect");

    alice
        .fetch_prekeys(bob_id, None)
        .await
        .expect("Can fetch bob keys");
    let status = alice
        .send_message(bob_id, "Hello bob!")
        .await
        .expect("Alice can send message");

    bob.connect().await.expect("Bob can connect");

    let mut bob_recv = bob.subscribe();

    // give time for the websocket client to process and respond to server
    tokio::time::sleep(Duration::from_millis(300)).await;

    bob.process_messages()
        .await
        .expect("Bob can process messages");

    let res = timeout(Duration::from_millis(300), bob_recv.recv())
        .await
        .expect("Bob receives message in time")
        .expect("receiver works");

    let msg = String::from_utf8_lossy(res.content_bytes());

    assert!(matches!(status, MessageStatus::Ok));
    assert!(msg == "Hello bob!")
}

#[tokio::test]
async fn alice_send_to_bob_missing_devices() {
    let _ = env_logger::try_init();
    let address = "127.0.0.1:9181";
    let mut server = TestServer::start(address).await;
    server
        .started_rx()
        .await
        .expect("Should be able to start server");

    let mut alice = client(address, "alice", "alice device").await;
    let mut bob = client(address, "bob", "bob device").await;

    let token = bob
        .create_provision()
        .await
        .expect("bob can init provisioning");
    let bob_id = bob.account_id().await.expect("Bob can get account id");
    bob.disconnect().await.expect("bob can disconnect");

    alice
        .fetch_prekeys(bob_id, None)
        .await
        .expect("Can fetch bob keys");
    let _bob_device = client_device(
        address,
        "bob_device",
        bob.identity_key_pair().await.expect("can get id pair"),
        token,
    )
    .await;

    let status = alice
        .send_message(bob_id, "Hello bob!")
        .await
        .expect("Alice can send message");

    assert!(matches!(status, MessageStatus::MissingDevices(_)))
}

#[tokio::test]
async fn alice_send_to_bob_two_devices() {
    let address = "127.0.0.1:9181";
    let mut server = TestServer::start(address).await;
    server
        .started_rx()
        .await
        .expect("Should be able to start server");

    let mut alice = client(address, "alice", "alice device").await;
    let mut bob = client(address, "bob", "bob device").await;

    let token = bob
        .create_provision()
        .await
        .expect("bob can init provisioning");
    let bob_id = bob.account_id().await.expect("Bob can get account id");

    let mut bob_device = client_device(
        address,
        "bob_device",
        bob.identity_key_pair().await.expect("can get id pair"),
        token,
    )
    .await;
    alice
        .fetch_prekeys(bob_id, None)
        .await
        .expect("Can fetch bob keys");

    let status = alice
        .send_message(bob_id, "Hello bob!")
        .await
        .expect("Alice can send message");

    let mut bob_device_recv = bob_device.subscribe();
    let mut bob_recv = bob.subscribe();

    bob.process_messages()
        .await
        .expect("Bob can process messages");
    bob_device
        .process_messages()
        .await
        .expect("Bob device can process messages");

    let res = timeout(Duration::from_millis(300), bob_recv.recv())
        .await
        .expect("Bob receives message in time")
        .expect("receiver works");
    let bob_msg = String::from_utf8_lossy(res.content_bytes());
    let res = timeout(Duration::from_millis(300), bob_device_recv.recv())
        .await
        .expect("Bob receives message in time")
        .expect("receiver works");
    let bob_device_msg = String::from_utf8_lossy(res.content_bytes());

    assert!(matches!(status, MessageStatus::Ok));
    assert!(bob_msg == "Hello bob!");
    assert!(bob_device_msg == "Hello bob!");
}

#[tokio::test]
async fn alice_send_to_bob_needs_sync() {
    let address = "127.0.0.1:9181";
    let mut server = TestServer::start(address).await;
    server
        .started_rx()
        .await
        .expect("Should be able to start server");

    let mut alice = client(address, "alice", "alice device").await;
    let bob = client(address, "bob", "bob device").await;

    let token = alice
        .create_provision()
        .await
        .expect("bob can init provisioning");
    let bob_id = bob.account_id().await.expect("Bob can get account id");

    let mut _alice_device = client_device(
        address,
        "bob_device",
        alice.identity_key_pair().await.expect("can get id pair"),
        token,
    )
    .await;

    alice
        .fetch_prekeys(bob_id, None)
        .await
        .expect("Can fetch bob keys");
    let status = alice
        .send_message(bob_id, "Hello bob!")
        .await
        .expect("Alice can send message");

    assert!(matches!(status, MessageStatus::NeedsSync))
}
