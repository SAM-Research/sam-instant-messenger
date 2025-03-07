use crate::utils::server::TestServer;
use sam_client::net::http_client::HttpClientConfig;
use sam_client::net::protocol::WebSocketProtocolClientConfig;
use sam_client::storage::sqlite::SqliteStoreConfig;
use sam_client::Client;

mod utils;

/*
   Ports used: 9383
*/

#[tokio::test]
pub async fn alice_can_find_bobs_account_id() {
    let _ = env_logger::try_init();
    let address = "http://127.0.0.1:9383".to_owned();
    let mut server = TestServer::start("127.0.0.1:9383").await;

    server
        .started_rx()
        .await
        .expect("Should be able to start server");

    let alice = Client::from_registration()
        .username("Alice")
        .device_name("Alice's Device")
        .store_config(SqliteStoreConfig::in_memory().await)
        .api_client_config(HttpClientConfig::new(address.clone()))
        .protocol_config(WebSocketProtocolClientConfig::new(address.clone()))
        .call()
        .await
        .unwrap();

    let bob = Client::from_registration()
        .username("Bob")
        .device_name("Bob's Device")
        .store_config(SqliteStoreConfig::in_memory().await)
        .api_client_config(HttpClientConfig::new(address.clone()))
        .protocol_config(WebSocketProtocolClientConfig::new(address))
        .call()
        .await
        .unwrap();

    let result = alice.get_user_account_id("Bob").await;
    println!("{:?}", result);
    assert!(result.is_ok());
    assert_eq!(bob.account_id().await.unwrap(), result.unwrap())
}
