mod utils;

use std::time::Duration;

use libsignal_protocol::IdentityKeyPair;
use rstest::rstest;
use sam_client::{
    encryption::envelope::DecryptedEnvelope,
    net::{
        http_client::HttpClientConfig,
        protocol::{
            client::ProtocolClient, traits::SamProtocolClient, WebSocketProtocolClientConfig,
        },
        ApiClient, HttpClient,
    },
    storage::{
        sqlite::{SqliteStoreConfig, SqliteStoreType},
        StoreType,
    },
    Client, ClientError,
};
use sam_common::{api::LinkDeviceToken, AccountId};
use tempfile::NamedTempFile;
use tokio::{sync::broadcast::Receiver, time::timeout};

use crate::utils::server::TestServer;
use crate::utils::tls::{make_rustls_client_config, make_rustls_server_config};

const TIMEOUT_SECS: u64 = 120;

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
        .upload_prekey_count(5)
        .call()
        .await
        .expect("Can register Client")
}

async fn tls_client(
    address: &str,
    username: &str,
    device_name: &str,
) -> Client<SqliteStoreType, HttpClient, ProtocolClient> {
    let client_config =
        make_rustls_client_config("./cert/rootCA.crt").expect("Should make client config");
    Client::from_registration()
        .username(username)
        .device_name(device_name)
        .store_config(SqliteStoreConfig::in_memory().await)
        .api_client_config(HttpClientConfig::new_with_tls(
            address.to_string(),
            client_config.clone(),
        ))
        .protocol_config(WebSocketProtocolClientConfig::new_with_tls(
            address.to_string(),
            client_config,
        ))
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
) -> Client<SqliteStoreType, HttpClient, ProtocolClient> {
    Client::from_provisioning()
        .store_config(SqliteStoreConfig::in_memory().await)
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
   PORTS USED: 9180-9190
*/

#[tokio::test]
async fn test_alice_send_to_bob_offline() {
    timeout(Duration::from_secs(TIMEOUT_SECS), async {
        let address = "127.0.0.1:9181";
        let mut server = TestServer::start(address, None).await;
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
        let mut server = TestServer::start(address, None).await;
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
async fn test_alice_send_to_bob_missing_devices() {
    timeout(Duration::from_secs(TIMEOUT_SECS), async {
        let address = "127.0.0.1:9184";
        let mut server = TestServer::start(address, None).await;
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

        alice
            .send_message(bob_id, "Hello bob!")
            .await
            .expect("Alice can send message");

        let _bob_device = client_device(
            address,
            "bob_device",
            bob.identity_key_pair().await.expect("can get id pair"),
            token,
        )
        .await;

        assert!(matches!(
            alice.send_message(bob_id, "Hello again, Bob").await,
            Err(ClientError::MissingDevices)
        ));

        assert!(alice.send_message(bob_id, "Hello again, Bob").await.is_ok());
    })
    .await
    .expect("Test took to long to complete")
}

#[tokio::test]
async fn test_alice_send_to_bob_extra_devices() {
    timeout(Duration::from_secs(TIMEOUT_SECS), async {
        let address = "127.0.0.1:9185";
        let mut server = TestServer::start(address, None).await;
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

        assert!(
            bob_device.delete_device().await.is_ok(),
            "Bob can delete his device"
        );

        alice
            .send_message(bob_id, "Hello again, Bob")
            .await
            .expect("Alice can send again");

        bob.process_messages_blocking()
            .await
            .expect("Bob can process messages again");

        let res = bob_recv.recv().await.expect("receiver works");
        let bob_msg = String::from_utf8_lossy(res.content_bytes());

        assert!(bob_msg == "Hello again, Bob");
    })
    .await
    .expect("Test took to long to complete")
}

#[tokio::test]
async fn test_alice_send_to_bob_and_self() {
    timeout(Duration::from_secs(TIMEOUT_SECS), async {
        let address = "127.0.0.1:9186";
        let mut server = TestServer::start(address, None).await;
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

        let bob_expected = "Hello bob!";
        let res = alice.send_message(bob_id, bob_expected).await;
        assert!(matches!(res, Err(ClientError::MissingDevices)));

        alice
            .send_message(bob_id, bob_expected)
            .await
            .expect("Alice can send message");

        let mut alice_recv = alice_device.subscribe();

        alice_device
            .process_messages_blocking()
            .await
            .expect("Alice device can process message");

        let res = alice_recv.recv().await.expect("receiver works");
        let bob_msg = String::from_utf8_lossy(res.content_bytes());

        assert!(bob_msg == bob_expected)
    })
    .await
    .expect("Test took to long to complete")
}

#[tokio::test]
async fn test_alice_send_to_bob_with_tls() {
    timeout(Duration::from_secs(TIMEOUT_SECS), async {
        let _ = rustls::crypto::ring::default_provider().install_default();
        let address = "127.0.0.1:9187";
        let server_config = make_rustls_server_config("./cert/server.crt", "./cert/server.key");
        let mut server = TestServer::start(address, Some(server_config)).await;
        server
            .started_rx()
            .await
            .expect("Should be able to start server");

        let mut alice = tls_client(address, "alice", "alice device").await;
        let mut bob = tls_client(address, "bob", "bob device").await;

        let bob_id = bob.account_id().await.expect("Bob can get his id");

        let mut bob_recv = bob.subscribe();

        alice
            .send_message(bob_id, "Hello bob!")
            .await
            .expect("Alice can send message");

        bob.process_messages_blocking()
            .await
            .expect("Bob can process messages");

        let res = bob_recv.recv().await.expect("receiver works");

        let msg = String::from_utf8_lossy(res.content_bytes());

        assert!(msg == "Hello bob!")
    })
    .await
    .expect("Test took to long to complete")
}

enum Message<'a> {
    Alice(&'a str),
    Bob(&'a str),
}

async fn send(
    sender: &mut Client<impl StoreType, impl ApiClient, impl SamProtocolClient>,
    receiver: &mut Client<impl StoreType, impl ApiClient, impl SamProtocolClient>,
    id: AccountId,
    subscriber: &mut Receiver<DecryptedEnvelope>,
    msg: &str,
) {
    sender
        .send_message(id, msg)
        .await
        .expect("Sender client can send message");
    receiver
        .process_messages_blocking()
        .await
        .expect("Receiver client can process messages");
    let envelope = subscriber
        .recv()
        .await
        .expect("Subscriber can receive message");
    let decrypted = String::from_utf8_lossy(envelope.content_bytes());
    assert!(decrypted == msg);
}

#[rstest]
#[case(vec![Message::Alice("a"), Message::Bob("b"), Message::Alice("aa"), Message::Alice("aaa"), Message::Bob("bb")], "9088")]
#[case(vec![Message::Alice("a"), Message::Alice("aa"), Message::Alice("aaa"), Message::Bob("b"), Message::Bob("bb")], "9089")]
#[case(vec![Message::Bob("b"), Message::Alice("a")], "9080")]
#[tokio::test]
async fn test_ongoing_communication<'a>(#[case] sequence: Vec<Message<'a>>, #[case] port: &str) {
    timeout(Duration::from_secs(TIMEOUT_SECS), async {
        let address = format!("127.0.0.1:{port}");
        let mut server = TestServer::start(&address, None).await;
        server
            .started_rx()
            .await
            .expect("Should be able to start server");

        let mut alice = client(&address, "alice", "alice device").await;
        let mut bob = client(&address, "bob", "bob device").await;

        let alice_id = alice.account_id().await.expect("Alice can get id");
        let bob_id = bob.account_id().await.expect("Bob can get id");

        let mut alice_recv = alice.subscribe();
        let mut bob_recv = bob.subscribe();

        for message in sequence {
            match message {
                Message::Alice(msg) => {
                    send(&mut alice, &mut bob, bob_id, &mut bob_recv, msg).await;
                }
                Message::Bob(msg) => {
                    send(&mut bob, &mut alice, alice_id, &mut alice_recv, msg).await;
                }
            }
        }
    })
    .await
    .expect("Test took to long to complete")
}

#[tokio::test]
async fn sqlite_stores_alice_send_to_bob() {
    timeout(Duration::from_secs(TIMEOUT_SECS), async {
        let address = "127.0.0.1:9190";
        let mut server = TestServer::start(address, None).await;
        server
            .started_rx()
            .await
            .expect("Should be able to start server");

        let temp = NamedTempFile::new().expect("Can create tempfile");
        let path = format!("sqlite://{}?mode=rwc", temp.path().to_string_lossy());

        let mut alice = Client::from_registration()
            .username("Alice")
            .device_name("Alice's Device")
            .store_config(SqliteStoreConfig::new(path.clone()))
            .protocol_config(WebSocketProtocolClientConfig::new(address.to_owned()))
            .api_client_config(HttpClientConfig::new(address.to_owned()))
            .call()
            .await
            .expect("can register alice");

        alice.disconnect().await.expect("can disconnect alice");
        drop(alice);

        let store = SqliteStoreConfig::new(path)
            .load()
            .await
            .expect("can create a store");

        let mut alice = Client::from_store()
            .store(store)
            .protocol_config(WebSocketProtocolClientConfig::new(address.to_owned()))
            .api_client_config(HttpClientConfig::new(address.to_owned()))
            .call()
            .await
            .unwrap();

        let bob = Client::from_registration()
            .username("Bob")
            .device_name("Bob's Device")
            .store_config(SqliteStoreConfig::new("sqlite::memory:".to_owned()))
            .protocol_config(WebSocketProtocolClientConfig::new(address.to_owned()))
            .api_client_config(HttpClientConfig::new(address.to_owned()))
            .call()
            .await
            .unwrap();

        alice
            .send_message(bob.account_id().await.unwrap(), "Hello")
            .await
            .unwrap();
    })
    .await
    .expect("Test took to long to complete")
}
