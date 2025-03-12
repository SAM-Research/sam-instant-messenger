use crate::utils::server::TestServer;
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
use sam_server::server::CertificatePaths;

mod utils;

/*
   Ports used: 9383
*/
pub async fn register_alice(
    address: String,
) -> Client<SqliteStoreType, HttpClient, ProtocolClient> {
    Client::from_registration()
        .username("Alice")
        .device_name("Alice's Device")
        .store_config(SqliteStoreConfig::in_memory().await)
        .api_client_config(HttpClientConfig::new(address.clone(), None))
        .protocol_config(WebSocketProtocolClientConfig::new(address))
        .call()
        .await
        .expect("Can register Alice")
}

#[tokio::test]
pub async fn can_delete_a_client() {
    let _ = env_logger::try_init();
    let address = "127.0.0.1:9480".to_owned();
    let mut server = TestServer::start("127.0.0.1:9480", None).await;

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

#[tokio::test]
pub async fn cannot_delete_a_client_that_does_not_exist() {
    let _ = env_logger::try_init();
    let address = "127.0.0.1:9481".to_owned();
    let paths = CertificatePaths {
        key: "./cert/server.key".to_string(),
        cert: "./cert/server.crt".to_string(),
    };
    let mut server = TestServer::start("127.0.0.1:9481", Some(paths)).await;

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

    let api_client = HttpClientConfig::new(address.clone(), Some("./cert/rootCA.crt".to_string()));
    let protocol_client = WebSocketProtocolClientConfig::new(address);

    let client = Client::from_store()
        .store(store)
        .api_client_config(api_client)
        .protocol_config(protocol_client)
        .call()
        .await
        .expect("Can create client from store");

    let result = client.delete_account().await;
    assert!(result.is_err_and(|(_, err)| matches!(err, ClientError::Api(_))))
}

#[tokio::test]
pub async fn alice_can_find_bobs_account_id() {
    let _ = env_logger::try_init();
    let address = "127.0.0.1:9383".to_owned();
    let paths = CertificatePaths {
        key: "./cert/server.key".to_string(),
        cert: "./cert/server.crt".to_string(),
    };
    let mut server = TestServer::start("127.0.0.1:9383", Some(paths)).await;

    server
        .started_rx()
        .await
        .expect("Should be able to start server");

    let alice = Client::from_registration()
        .username("Alice")
        .device_name("Alice's Device")
        .store_config(SqliteStoreConfig::in_memory().await)
        .api_client_config(HttpClientConfig::new(
            address.clone(),
            Some("./cert/rootCA.crt".to_string()),
        ))
        .protocol_config(WebSocketProtocolClientConfig::new(address.clone()))
        .call()
        .await
        .unwrap();

    let bob = Client::from_registration()
        .username("Bob")
        .device_name("Bob's Device")
        .store_config(SqliteStoreConfig::in_memory().await)
        .api_client_config(HttpClientConfig::new(
            address.clone(),
            Some("./cert/rootCA.crt".to_string()),
        ))
        .protocol_config(WebSocketProtocolClientConfig::new(address))
        .call()
        .await
        .unwrap();

    let result = alice.get_account_id_for("Bob").await;

    assert!(result.is_ok());
    assert_eq!(bob.account_id().await.unwrap(), result.unwrap())
}
