use crate::utils::server::TestServer;

use crate::utils::tls::{make_rustls_client_config, make_rustls_server_config};
use libsignal_protocol::IdentityKeyPair;
use rand::rngs::OsRng;
use sam_client::{
    net::{
        http_client::HttpClientConfig,
        protocol::{client::ProtocolClient, WebSocketProtocolClientConfig},
        HttpClient,
    },
    storage::{
        sqlite::{SqliteStoreConfig, SqliteStoreType},
        AccountStore, StoreConfig,
    },
    Client, ClientError,
};
use sam_common::{address::RegistrationId, AccountId};

mod utils;

/*
   Ports used: 937x
*/
pub async fn register_alice(
    address: String,
) -> Result<Client<SqliteStoreType, HttpClient, ProtocolClient>, ClientError> {
    Client::from_registration()
        .username("Alice")
        .device_name("Alice's Device")
        .store_config(SqliteStoreConfig::in_memory().await)
        .api_client_config(HttpClientConfig::new(address.clone()))
        .protocol_config(WebSocketProtocolClientConfig::new(address))
        .call()
        .await
}

#[tokio::test]
pub async fn one_client_can_register() {
    let address = "127.0.0.1:9370".to_owned();
    let mut server = TestServer::start(&address, None).await;

    server
        .started_rx()
        .await
        .expect("Should be able to start server");

    let client = register_alice(address).await;

    assert!(client.is_ok());
}

#[tokio::test]
pub async fn can_delete_account() {
    let address = "127.0.0.1:9371".to_owned();
    let mut server = TestServer::start(&address, None).await;

    server
        .started_rx()
        .await
        .expect("Should be able to start server");

    let client = register_alice(address).await.expect("Can register account");

    assert!(client.delete_account().await.is_ok());
}

#[tokio::test]
pub async fn cannot_create_client_without_valid_account() {
    let address = "127.0.0.1:9372".to_owned();
    let mut server = TestServer::start(&address, None).await;

    server
        .started_rx()
        .await
        .expect("Should be able to start server");

    let mut csprng = OsRng;
    let key_pair = IdentityKeyPair::generate(&mut csprng);
    let registration_id = RegistrationId::generate(&mut csprng);
    let mut store = SqliteStoreConfig::in_memory()
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
    let protocol_client = WebSocketProtocolClientConfig::new(address);

    let client = Client::from_store()
        .store(store)
        .api_client_config(api_client)
        .protocol_config(protocol_client)
        .call()
        .await;

    assert!(client.is_err_and(|e| matches!(e, ClientError::Protocol(_))))
}

#[tokio::test]
pub async fn can_delete_a_device() {
    let address = "127.0.0.1:9373".to_owned();
    let mut server = TestServer::start(&address, None).await;

    server
        .started_rx()
        .await
        .expect("Should be able to start server");

    let client = register_alice(address).await.expect("Can register account");

    let result = client.delete_account().await;
    assert!(
        result.is_ok(),
        "Error deleting account: {:?}",
        result.unwrap_err().1
    )
}

#[tokio::test]
pub async fn alice_can_find_bobs_account_id() {
    let address = "127.0.0.1:9374".to_owned();
    let mut server = TestServer::start(&address, None).await;

    server
        .started_rx()
        .await
        .expect("Should be able to start server");

    let alice = register_alice(address.clone())
        .await
        .expect("Can create account");

    let bob = Client::from_registration()
        .username("Bob")
        .device_name("Bob's Device")
        .store_config(SqliteStoreConfig::in_memory().await)
        .api_client_config(HttpClientConfig::new(address.clone()))
        .protocol_config(WebSocketProtocolClientConfig::new(address))
        .call()
        .await
        .unwrap();

    let result = alice.get_account_id_for("Bob").await;

    assert!(result.is_ok());
    assert_eq!(bob.account_id().await.unwrap(), result.unwrap())
}

#[tokio::test]
pub async fn one_client_can_register_with_tls() {
    let _ = rustls::crypto::ring::default_provider().install_default();
    let address = "127.0.0.1:9375".to_owned();
    let server_config = make_rustls_server_config("./cert/server.crt", "./cert/server.key");
    let client_config = make_rustls_client_config("./cert/rootCA.crt").expect("Can make config");
    let mut server = TestServer::start(&address, Some(server_config)).await;

    server
        .started_rx()
        .await
        .expect("Should be able to start server");

    let client = Client::from_registration()
        .username("Alice")
        .device_name("Alice's Device")
        .store_config(SqliteStoreConfig::in_memory().await)
        .api_client_config(HttpClientConfig::new_with_tls(
            address.clone(),
            client_config.clone(),
        ))
        .protocol_config(WebSocketProtocolClientConfig::new_with_tls(
            address,
            client_config.clone(),
        ))
        .call()
        .await;

    assert!(client.is_ok());
}

#[tokio::test]
pub async fn two_clients_cannot_have_the_same_username() {
    let address = "127.0.0.1:9376".to_owned();
    let mut server = TestServer::start(&address, None).await;

    server
        .started_rx()
        .await
        .expect("Should be able to start server");

    let _alice = Client::from_registration()
        .username("Alice")
        .device_name("Alice's Device")
        .store_config(SqliteStoreConfig::in_memory().await)
        .api_client_config(HttpClientConfig::new(address.clone()))
        .protocol_config(WebSocketProtocolClientConfig::new(address.clone()))
        .call()
        .await
        .expect("Can make Alice");

    let alice_2 = Client::from_registration()
        .username("Alice")
        .device_name("Alice's Device")
        .store_config(SqliteStoreConfig::in_memory().await)
        .api_client_config(HttpClientConfig::new(address.clone()))
        .protocol_config(WebSocketProtocolClientConfig::new(address))
        .call()
        .await;

    assert!(alice_2.is_err());
}
