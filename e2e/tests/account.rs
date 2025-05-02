use libsignal_protocol::IdentityKeyPair;
use rand::rngs::OsRng;
use rstest::rstest;
use sam_client::{
    client::SqliteClientType,
    net::{http_client::HttpClientConfig, protocol::WebSocketProtocolClientConfig},
    storage::{sqlite::SqliteStoreConfig, AccountStore, StoreConfig},
    Client, ClientError,
};
use sam_common::{address::RegistrationId, AccountId};
use sam_net::tls::{create_tls_client_config, create_tls_server_config};
use sam_server::{ServerState, StateType};
use sam_test_utils::{
    e2e::{in_memory_server_state, postgres_server_state, TestServer},
    get_next_port,
};
use uuid::Uuid;

pub async fn register_someone(address: String) -> Result<Client<SqliteClientType>, ClientError> {
    let name = Uuid::new_v4().to_string();
    Client::from_registration()
        .username(&name)
        .device_name("Alice's Device")
        .store_config(SqliteStoreConfig::in_memory(10).await)
        .api_client_config(HttpClientConfig::new(address.clone()))
        .protocol_config(WebSocketProtocolClientConfig::new(address, 10))
        .call()
        .await
}

#[tokio::test]
#[rstest]
#[ignore = "requires a postgres test database"]
#[case::postgres(postgres_server_state())]
#[case::in_memory(in_memory_server_state())]
pub async fn one_client_can_register(
    #[future]
    #[case]
    state: ServerState<impl StateType>,
) {
    let address = format!("127.0.0.1:{}", get_next_port());
    let mut server = TestServer::start(&address, None, state.await).await;

    server
        .started_rx()
        .await
        .expect("Should be able to start server");

    let client = register_someone(address).await;

    assert!(client.is_ok());
}

#[tokio::test]
#[rstest]
#[ignore = "requires a postgres test database"]
#[case::postgres(postgres_server_state())]
#[case::in_memory(in_memory_server_state())]
pub async fn can_delete_account(
    #[future]
    #[case]
    state: ServerState<impl StateType>,
) {
    let address = format!("127.0.0.1:{}", get_next_port());
    let mut server = TestServer::start(&address, None, state.await).await;

    server
        .started_rx()
        .await
        .expect("Should be able to start server");

    let client = register_someone(address)
        .await
        .expect("Can register account");

    assert!(client.delete_account().await.is_ok());
}

#[tokio::test]
#[rstest]
#[ignore = "requires a postgres test database"]
#[case::postgres(postgres_server_state())]
#[case::in_memory(in_memory_server_state())]
pub async fn cannot_create_client_without_valid_account(
    #[future]
    #[case]
    state: ServerState<impl StateType>,
) {
    let address = format!("127.0.0.1:{}", get_next_port());
    let mut server = TestServer::start(&address, None, state.await).await;

    server
        .started_rx()
        .await
        .expect("Should be able to start server");

    let mut csprng = OsRng;
    let key_pair = IdentityKeyPair::generate(&mut csprng);
    let registration_id = RegistrationId::generate(&mut csprng);
    let mut store = SqliteStoreConfig::in_memory(10)
        .await
        .create_store(key_pair, registration_id)
        .await
        .expect("can create in-memory store");

    store
        .account_store
        .set_account_id(AccountId::generate())
        .await
        .expect("Can set account_id");

    store
        .account_store
        .set_device_id(1.into())
        .await
        .expect("Can set device_id");

    store
        .account_store
        .set_password("Alice's Password".to_owned())
        .await
        .expect("Can set password");

    let api_client = HttpClientConfig::new(address.clone());
    let protocol_client = WebSocketProtocolClientConfig::new(address, 10);

    let client: Result<Client<SqliteClientType>, _> = Client::from_store()
        .store(store)
        .api_client_config(api_client)
        .protocol_config(protocol_client)
        .call()
        .await;

    assert!(client.is_err_and(|e| matches!(e, ClientError::Protocol(_))))
}

#[tokio::test]
#[rstest]
#[ignore = "requires a postgres test database"]
#[case::postgres(postgres_server_state())]
#[case::in_memory(in_memory_server_state())]
pub async fn can_delete_a_device(
    #[future]
    #[case]
    state: ServerState<impl StateType>,
) {
    let address = format!("127.0.0.1:{}", get_next_port());
    let mut server = TestServer::start(&address, None, state.await).await;

    server
        .started_rx()
        .await
        .expect("Should be able to start server");

    let client = register_someone(address)
        .await
        .expect("Can register account");

    let result = client.delete_account().await;
    assert!(
        result.is_ok(),
        "Error deleting account: {:?}",
        result.unwrap_err().1
    )
}

#[tokio::test]
#[rstest]
#[ignore = "requires a postgres test database"]
#[case::postgres(postgres_server_state())]
#[case::in_memory(in_memory_server_state())]
pub async fn alice_can_find_bobs_account_id(
    #[future]
    #[case]
    state: ServerState<impl StateType>,
) {
    let address = format!("127.0.0.1:{}", get_next_port());
    let mut server = TestServer::start(&address, None, state.await).await;

    server
        .started_rx()
        .await
        .expect("Should be able to start server");

    let alice = register_someone(address.clone())
        .await
        .expect("Can create account");

    let bob_username = Uuid::new_v4().to_string();
    let bob: Client<SqliteClientType> = Client::from_registration()
        .username(&bob_username)
        .device_name("Bob's Device")
        .store_config(SqliteStoreConfig::in_memory(10).await)
        .api_client_config(HttpClientConfig::new(address.clone()))
        .protocol_config(WebSocketProtocolClientConfig::new(address, 10))
        .call()
        .await
        .unwrap();

    let result = alice.get_account_id_for(&bob_username).await;

    assert!(result.is_ok());
    assert_eq!(bob.account_id(), result.unwrap())
}

#[tokio::test]
#[rstest]
#[ignore = "requires a postgres test database"]
#[case::postgres(postgres_server_state())]
#[case::in_memory(in_memory_server_state())]
pub async fn one_client_can_register_with_tls(
    #[future]
    #[case]
    state: ServerState<impl StateType>,
) {
    let _ = rustls::crypto::ring::default_provider().install_default();
    let address = format!("127.0.0.1:{}", get_next_port());
    let server_config = create_tls_server_config("./cert/server.crt", "./cert/server.key", None)
        .expect("Can create server config");
    let client_config =
        create_tls_client_config("./cert/rootCA.crt", None).expect("Can create client config");
    let mut server = TestServer::start(&address, Some(server_config), state.await).await;

    server
        .started_rx()
        .await
        .expect("Should be able to start server");

    let client: Result<Client<SqliteClientType>, _> = Client::from_registration()
        .username(&Uuid::new_v4().to_string())
        .device_name("Alice's Device")
        .store_config(SqliteStoreConfig::in_memory(10).await)
        .api_client_config(HttpClientConfig::new_with_tls(
            address.clone(),
            client_config.clone(),
        ))
        .protocol_config(WebSocketProtocolClientConfig::new_with_tls(
            address,
            client_config.clone(),
            10,
        ))
        .call()
        .await;

    assert!(client.is_ok());
}

#[tokio::test]
#[rstest]
#[ignore = "requires a postgres test database"]
#[case::postgres(postgres_server_state())]
#[case::in_memory(in_memory_server_state())]
pub async fn two_clients_cannot_have_the_same_username(
    #[future]
    #[case]
    state: ServerState<impl StateType>,
) {
    let address = format!("127.0.0.1:{}", get_next_port());
    let mut server = TestServer::start(&address, None, state.await).await;

    let username = Uuid::new_v4().to_string();

    server
        .started_rx()
        .await
        .expect("Should be able to start server");

    let _alice: Client<SqliteClientType> = Client::from_registration()
        .username(&username)
        .device_name("Device")
        .store_config(SqliteStoreConfig::in_memory(10).await)
        .api_client_config(HttpClientConfig::new(address.clone()))
        .protocol_config(WebSocketProtocolClientConfig::new(address.clone(), 10))
        .call()
        .await
        .expect("Can make the first client");

    let alice_2: Result<Client<SqliteClientType>, _> = Client::from_registration()
        .username(&username)
        .device_name("Device")
        .store_config(SqliteStoreConfig::in_memory(10).await)
        .api_client_config(HttpClientConfig::new(address.clone()))
        .protocol_config(WebSocketProtocolClientConfig::new(address, 10))
        .call()
        .await;

    assert!(alice_2.is_err());
}
