use crate::utils::client::registration_request;
use crate::utils::server::TestServer;
use libsignal_protocol::IdentityKeyPair;
use rand::rngs::OsRng;
use sam_client::net::http_client::HttpClientConfig;
use sam_client::net::protocol::WebSocketProtocolClientConfig;
use sam_client::net::{ApiClient, HttpClient};
use sam_client::storage::sqlite::SqliteStoreConfig;
use sam_client::Client;
use sam_server::server::CertificatePaths;

mod utils;

/*
   PORTS USED: 9381-9382
*/

#[tokio::test]
pub async fn one_client_can_register() {
    let _ = env_logger::try_init();
    let address = "127.0.0.1:9381".to_owned();
    let paths = CertificatePaths {
        key: "./cert/server.key".to_string(),
        cert: "./cert/server.crt".to_string(),
    };
    let mut server = TestServer::start("127.0.0.1:9381", Some(paths)).await;

    server
        .started_rx()
        .await
        .expect("Should be able to start server");

    let client = Client::from_registration()
        .username("Alice")
        .device_name("Alice's Device")
        .store_config(SqliteStoreConfig::in_memory().await)
        .api_client_config(HttpClientConfig::new(
            address.clone(),
            Some("./cert/rootCA.crt".to_string()),
        ))
        .protocol_config(WebSocketProtocolClientConfig::new(address))
        .call()
        .await;

    assert!(client.is_ok());
}

#[tokio::test]
pub async fn can_delete_account() {
    let _ = env_logger::try_init();
    let address = "http://127.0.0.1:9382";
    let mut server = TestServer::start("127.0.0.1:9382", None).await;
    let mut csprng = OsRng;
    let id_key_pair = IdentityKeyPair::generate(&mut csprng);

    server
        .started_rx()
        .await
        .expect("Should be able to start server");

    let client = HttpClient::new(address.to_owned());

    let result = client
        .register_account("Alice", "Alice Password", registration_request(id_key_pair))
        .await;

    assert!(result.is_ok());

    let account_id = result.unwrap().account_id;

    assert!(client
        .delete_account(account_id, 1.into(), "Alice Password")
        .await
        .inspect_err(|err| println!("{err}"))
        .is_ok());
}
