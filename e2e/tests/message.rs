mod utils;

use std::time::Duration;

use libsignal_protocol::IdentityKeyPair;
use sam_client::{
    net::{
        http_client::HttpClientConfig,
        protocol::{client::ProtocolClient, WebSocketProtocolClientConfig},
        HttpClient,
    },
    storage::inmem::{InMemoryStoreConfig, InMemoryStoreType},
    Client,
};
use sam_common::api::LinkDeviceToken;
use tokio::time::timeout;

use crate::utils::server::TestServer;

const TIMEOUT_SECS: u64 = 120;

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
        .upload_prekey_count(5)
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
        .upload_prekey_count(5)
        .call()
        .await
        .expect("Can provision device")
}

/*
   PORTS USED: 9180-9189
*/

#[tokio::test]
async fn test_alice_send_to_bob() {
    timeout(Duration::from_secs(TIMEOUT_SECS), async {
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
            .send_message(bob_id, "Hello bob!")
            .await
            .expect("Alice can send message");

        bob.process_messages()
            .await
            .expect("Bob can process messages");

        let res = bob_recv.recv().await.expect("receiver works");

        let msg = String::from_utf8_lossy(res.content_bytes());

        assert!(msg == "Hello bob!")
    })
    .await
    .expect("Test took to long to complete")
}

#[tokio::test]
async fn test_alice_send_to_bob_offline() {
    timeout(Duration::from_secs(TIMEOUT_SECS), async {
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
            .send_message(bob_id, "Hello bob!")
            .await
            .expect("Alice can send message");

        bob.connect().await.expect("Bob can connect");

        let mut bob_recv = bob.subscribe();

        bob.process_messages_blocking()
            .await
            .expect("Bob processes messages");

        let res = bob_recv.recv().await.expect("receiver works");

        let msg = String::from_utf8_lossy(res.content_bytes());

        assert!(msg == "Hello bob!")
    })
    .await
    .expect("Test took to long to complete")
}

#[tokio::test]
async fn test_alice_send_to_bob_two_devices() {
    timeout(Duration::from_secs(TIMEOUT_SECS), async {
        let address = "127.0.0.1:9183";
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
            .send_message(bob_id, "Hello bob!")
            .await
            .expect("Alice can send message");

        let mut bob_device_recv = bob_device.subscribe();
        let mut bob_recv = bob.subscribe();

        bob.process_messages_blocking()
            .await
            .expect("Bob can process messages");
        bob_device
            .process_messages_blocking()
            .await
            .expect("Bob device can process messages");

        let res = bob_recv.recv().await.expect("receiver works");
        let bob_msg = String::from_utf8_lossy(res.content_bytes());

        let res = bob_device_recv.recv().await.expect("receiver works");
        let bob_device_msg = String::from_utf8_lossy(res.content_bytes());

        assert!(bob_msg == "Hello bob!");
        assert!(bob_device_msg == "Hello bob!");
    })
    .await
    .expect("Test took to long to complete")
}

#[tokio::test]
async fn test_alice_send_to_bob_and_self() {
    timeout(Duration::from_secs(TIMEOUT_SECS), async {
        let address = "127.0.0.1:9184";
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

        let mut alice_device = client_device(
            address,
            "bob_device",
            alice.identity_key_pair().await.expect("can get id pair"),
            token,
        )
        .await;

        alice
            .send_message(bob_id, "Hello bob!")
            .await
            .expect("Alice can send message");

        let mut alice_recv = alice_device.subscribe();

        alice_device
            .process_messages_blocking()
            .await
            .expect("Alice device can process message");

        let res = alice_recv.recv().await.expect("receiver works");
        let bob_msg = String::from_utf8_lossy(res.content_bytes());

        assert!(bob_msg == "Hello bob!")
    })
    .await
    .expect("Test took to long to complete")
}
