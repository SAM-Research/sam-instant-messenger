use libsignal_protocol::IdentityKeyPair;
use rstest::rstest;
use sam_client::{
    client::SqliteClientType,
    encryption::envelope::DecryptedEnvelope,
    logic::LogicError,
    net::{http_client::HttpClientConfig, protocol::WebSocketProtocolClientConfig},
    storage::sqlite::{sqlite_connector::SqliteConnector, SqliteStoreConfig},
    Client, ClientError,
};
use sam_common::{api::LinkDeviceToken, AccountId};
use sam_net::tls::{create_tls_client_config, MutualTlsConfig};
use sam_server::config::TlsConfig;
use sam_server::{ServerState, StateType};
use sam_test_utils::{
    e2e::{in_memory_server_state, postgres_server_state, TestServer},
    get_next_port,
};
use std::time::Duration;
use tempfile::NamedTempFile;
use tokio::{sync::broadcast::Receiver, time::timeout};
use uuid::Uuid;

const TIMEOUT_SECS: u64 = 120;

async fn client(address: &str, device_name: &str) -> Client<SqliteClientType> {
    let username = Uuid::new_v4().to_string();
    println!("User {}, with device {}", &username, &device_name);
    Client::from_registration()
        .username(&username)
        .device_name(device_name)
        .store_config(
            SqliteStoreConfig::in_memory(10)
                .await
                .expect("can create inmemory"),
        )
        .api_client_config(HttpClientConfig::new(address.to_string()))
        .protocol_config(WebSocketProtocolClientConfig::new(address.to_string(), 10))
        .upload_prekey_count(5)
        .call()
        .await
        .expect("Can register Client")
}

async fn tls_client(
    address: &str,
    device_name: &str,
    mutual_config: Option<MutualTlsConfig>,
) -> Client<SqliteClientType> {
    let client_config = create_tls_client_config("./cert/rootCA.crt", mutual_config)
        .expect("Can create client config");
    let username = Uuid::new_v4().to_string();
    Client::from_registration()
        .username(&username)
        .device_name(device_name)
        .store_config(
            SqliteStoreConfig::in_memory(10)
                .await
                .expect("can create inmemory"),
        )
        .api_client_config(HttpClientConfig::new_with_tls(
            address.to_string(),
            client_config.clone(),
        ))
        .protocol_config(WebSocketProtocolClientConfig::new_with_tls(
            address.to_string(),
            client_config,
            10,
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
) -> Client<SqliteClientType> {
    Client::from_provisioning()
        .store_config(
            SqliteStoreConfig::in_memory(10)
                .await
                .expect("can create inmemory"),
        )
        .api_client_config(HttpClientConfig::new(address.to_string()))
        .protocol_config(WebSocketProtocolClientConfig::new(address.to_string(), 10))
        .device_name(device_name)
        .id_key_pair(id_pair)
        .token(token)
        .upload_prekey_count(5)
        .call()
        .await
        .expect("Can provision device")
}

#[tokio::test]
#[rstest]
#[ignore = "requires a postgres test database"]
#[case::postgres(postgres_server_state())]
#[case::in_memory(in_memory_server_state())]
async fn test_alice_send_to_bob_offline(
    #[future]
    #[case]
    state: ServerState<impl StateType>,
) {
    timeout(Duration::from_secs(TIMEOUT_SECS), async {
        let address = format!("127.0.0.1:{}", get_next_port());
        let mut server = TestServer::start(&address, None, state.await).await;
        server
            .started_rx()
            .await
            .expect("Should be able to start server");

        let mut alice = client(&address, "alice device").await;
        let mut bob = client(&address, "bob device").await;

        let bob_id = bob.account_id();

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
#[rstest]
#[ignore = "requires a postgres test database"]
#[case::postgres(postgres_server_state())]
#[case::in_memory(in_memory_server_state())]
async fn test_alice_send_to_bob_two_devices(
    #[future]
    #[case]
    state: ServerState<impl StateType>,
) {
    timeout(Duration::from_secs(TIMEOUT_SECS), async {
        let address = format!("127.0.0.1:{}", get_next_port());
        let mut server = TestServer::start(&address, None, state.await).await;
        server
            .started_rx()
            .await
            .expect("Should be able to start server");

        let mut alice = client(&address, "alice device").await;
        let mut bob = client(&address, "bob device").await;

        let token = bob
            .create_provision()
            .await
            .expect("bob can init provisioning");
        let bob_id = bob.account_id();

        let mut bob_device = client_device(
            &address,
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
#[rstest]
#[ignore = "requires a postgres test database"]
#[case::postgres(postgres_server_state())]
#[case::in_memory(in_memory_server_state())]
async fn test_alice_send_to_bob_missing_devices(
    #[future]
    #[case]
    state: ServerState<impl StateType>,
) {
    timeout(Duration::from_secs(TIMEOUT_SECS), async {
        let address = format!("127.0.0.1:{}", get_next_port());
        let mut server = TestServer::start(&address, None, state.await).await;
        server
            .started_rx()
            .await
            .expect("Should be able to start server");

        let mut alice = client(&address, "alice device").await;
        let mut bob = client(&address, "bob device").await;

        let token = bob
            .create_provision()
            .await
            .expect("bob can init provisioning");
        let bob_id = bob.account_id();

        alice
            .send_message(bob_id, "Hello bob!")
            .await
            .expect("Alice can send message");

        let _bob_device = client_device(
            &address,
            "bob_device",
            bob.identity_key_pair().await.expect("can get id pair"),
            token,
        )
        .await;

        assert!(matches!(
            alice.send_message(bob_id, "Hello again, Bob").await,
            Err(ClientError::Logic(LogicError::MissingDevices))
        ));

        assert!(alice.send_message(bob_id, "Hello again, Bob").await.is_ok());
    })
    .await
    .expect("Test took to long to complete")
}

#[tokio::test]
#[rstest]
#[ignore = "requires a postgres test database"]
#[case::postgres(postgres_server_state())]
#[case::in_memory(in_memory_server_state())]
async fn test_alice_send_to_bob_extra_devices(
    #[future]
    #[case]
    state: ServerState<impl StateType>,
) {
    timeout(Duration::from_secs(TIMEOUT_SECS), async {
        let address = format!("127.0.0.1:{}", get_next_port());
        let mut server = TestServer::start(&address, None, state.await).await;
        server
            .started_rx()
            .await
            .expect("Should be able to start server");

        let mut alice = client(&address, "alice device").await;
        let mut bob = client(&address, "bob device").await;

        let token = bob
            .create_provision()
            .await
            .expect("bob can init provisioning");
        let bob_id = bob.account_id();

        let mut bob_device = client_device(
            &address,
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

#[rstest]
#[ignore = "requires a postgres test database"]
#[case::postgres(postgres_server_state())]
#[case::in_memory(in_memory_server_state())]
#[tokio::test]
async fn test_alice_send_to_bob_and_self(
    #[future]
    #[case]
    state: ServerState<impl StateType>,
) {
    let _ = env_logger::try_init();
    timeout(Duration::from_secs(TIMEOUT_SECS), async {
        let address = format!("127.0.0.1:{}", get_next_port());
        let mut server = TestServer::start(&address, None, state.await).await;
        server
            .started_rx()
            .await
            .expect("Should be able to start server");

        let mut alice = client(&address, "alice device").await;
        let bob = client(&address, "bob device").await;

        let token = alice
            .create_provision()
            .await
            .expect("bob can init provisioning");
        let bob_id = bob.account_id();

        let mut alice_device = client_device(
            &address,
            "bob_device",
            alice.identity_key_pair().await.expect("can get id pair"),
            token,
        )
        .await;

        let bob_expected = "Hello bob!";
        let res = alice.send_message(bob_id, bob_expected).await;
        assert!(matches!(
            res,
            Err(ClientError::Logic(LogicError::MissingDevices))
        ));

        println!("Alice could not send to bob");

        alice
            .send_message(bob_id, bob_expected)
            .await
            .expect("Alice can send message");

        println!("Alice sent message");

        let mut alice_recv = alice_device.subscribe();

        println!("Alice is waiting for her message");
        alice_device
            .process_messages_blocking()
            .await
            .expect("Alice device can process message");

        println!("Alice processing messages");

        let res = alice_recv.recv().await.expect("receiver works");
        let bob_msg = String::from_utf8_lossy(res.content_bytes());

        assert!(bob_msg == bob_expected)
    })
    .await
    .expect("Test took to long to complete")
}

#[rstest]
#[ignore = "requires a postgres test database"]
#[case(Some("./cert/rootCA.crt".to_string()), Some(MutualTlsConfig::new("./cert/client.key".to_string(), "./cert/client.crt".to_string())), get_next_port(), postgres_server_state())]
#[ignore = "requires a postgres test database"]
#[case(None, None, get_next_port(), postgres_server_state())]
#[case(Some("./cert/rootCA.crt".to_string()), Some(MutualTlsConfig::new("./cert/client.key".to_string(), "./cert/client.crt".to_string())), get_next_port(), in_memory_server_state())]
#[case(None, None, get_next_port(), in_memory_server_state())]
#[tokio::test]
async fn test_alice_send_to_bob_with_tls(
    #[case] ca_cert: Option<String>,
    #[case] mutual_config: Option<MutualTlsConfig>,
    #[case] port: u16,
    #[future(awt)]
    #[case]
    state: ServerState<impl StateType>,
) {
    timeout(Duration::from_secs(TIMEOUT_SECS), async {
        let _ = rustls::crypto::ring::default_provider().install_default();
        let address = format!("127.0.0.1:{}", port);
        let server_config = TlsConfig {
            ca_cert_path: ca_cert,
            cert_path: "./cert/server.crt".to_string(),
            key_path: "./cert/server.key".to_string(),
        }
        .try_into()
        .expect("Can create server config");
        let mut server = TestServer::start(&address, Some(server_config), state).await;
        server
            .started_rx()
            .await
            .expect("Should be able to start server");

        let mut alice = tls_client(&address, "alice device", mutual_config.clone()).await;
        let mut bob = tls_client(&address, "bob device", mutual_config).await;

        let bob_id = bob.account_id();

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
    sender: &mut Client<SqliteClientType>,
    receiver: &mut Client<SqliteClientType>,
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
#[case(vec![Message::Alice("a"), Message::Bob("b"), Message::Alice("aa"), Message::Alice("aaa"), Message::Bob("bb")], get_next_port(), in_memory_server_state())]
#[case(vec![Message::Alice("a"), Message::Alice("aa"), Message::Alice("aaa"), Message::Bob("b"), Message::Bob("bb")], get_next_port(), in_memory_server_state())]
#[case(vec![Message::Bob("b"), Message::Alice("a")], get_next_port(), in_memory_server_state())]
#[ignore = "requires a postgres test database"]
#[case(vec![Message::Alice("a"), Message::Bob("b"), Message::Alice("aa"), Message::Alice("aaa"), Message::Bob("bb")], get_next_port(), postgres_server_state())]
#[ignore = "requires a postgres test database"]
#[case(vec![Message::Alice("a"), Message::Alice("aa"), Message::Alice("aaa"), Message::Bob("b"), Message::Bob("bb")], get_next_port(), postgres_server_state())]
#[ignore = "requires a postgres test database"]
#[case(vec![Message::Bob("b"), Message::Alice("a")], get_next_port(), postgres_server_state())]
#[tokio::test]
async fn test_ongoing_communication<'a>(
    #[case] sequence: Vec<Message<'a>>,
    #[case] port: u16,
    #[future(awt)]
    #[case]
    state: ServerState<impl StateType>,
) {
    timeout(Duration::from_secs(TIMEOUT_SECS), async {
        let address = format!("127.0.0.1:{}", port);
        let mut server = TestServer::start(&address, None, state).await;
        server
            .started_rx()
            .await
            .expect("Should be able to start server");

        let mut alice = client(&address, "alice device").await;
        let mut bob = client(&address, "bob device").await;

        let alice_id = alice.account_id();
        let bob_id = bob.account_id();

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

#[rstest]
#[ignore = "requires a postgres test database"]
#[case::postgres(postgres_server_state())]
#[case::in_memory(in_memory_server_state())]
#[tokio::test]
async fn sqlite_stores_alice_send_to_bob(
    #[future(awt)]
    #[case]
    state: ServerState<impl StateType>,
) {
    timeout(Duration::from_secs(TIMEOUT_SECS), async {
        let address = format!("127.0.0.1:{}", get_next_port());
        let mut server = TestServer::start(&address, None, state).await;
        server
            .started_rx()
            .await
            .expect("Should be able to start server");

        let temp = NamedTempFile::new().expect("Can create tempfile");
        let path = format!("sqlite://{}?mode=rwc", temp.path().to_string_lossy());

        let connector = SqliteConnector::migrate(&path)
            .await
            .expect("can connect to sqlite");
        let mut alice: Client<SqliteClientType> = Client::from_registration()
            .username(&Uuid::new_v4().to_string())
            .device_name("Alice's Device")
            .store_config(SqliteStoreConfig::new(connector, 10))
            .protocol_config(WebSocketProtocolClientConfig::new(address.to_owned(), 10))
            .api_client_config(HttpClientConfig::new(address.to_owned()))
            .call()
            .await
            .expect("can register alice");

        alice.disconnect().await.expect("can disconnect alice");
        drop(alice);

        let connector = SqliteConnector::migrate(&path)
            .await
            .expect("can connect to sqlite");
        let store = SqliteStoreConfig::new(connector, 10).into();

        let mut alice: Client<SqliteClientType> = Client::from_store()
            .store(store)
            .protocol_config(WebSocketProtocolClientConfig::new(address.to_owned(), 10))
            .api_client_config(HttpClientConfig::new(address.to_owned()))
            .call()
            .await
            .unwrap();

        let bob: Client<SqliteClientType> = Client::from_registration()
            .username(&Uuid::new_v4().to_string())
            .device_name("Bob's Device")
            .store_config(
                SqliteStoreConfig::in_memory(10)
                    .await
                    .expect("can create inmemory"),
            )
            .protocol_config(WebSocketProtocolClientConfig::new(address.to_owned(), 10))
            .api_client_config(HttpClientConfig::new(address.to_owned()))
            .call()
            .await
            .unwrap();

        alice.send_message(bob.account_id(), "Hello").await.unwrap();
    })
    .await
    .expect("Test took to long to complete")
}
