use sam_client::net::http_client::HttpClientConfig;
use sam_client::net::protocol::WebSocketProtocolClientConfig;
use sam_client::storage::sqlite::SqliteStoreConfig;
use sam_client::Client;

mod utils;

use crate::utils::server::TestServer;

/*
   PORTS USED: 9384-9386
*/

#[tokio::test]
pub async fn alice_can_upload_keys() {
    let _ = env_logger::try_init();
    let address = "127.0.0.1:9384".to_owned();
    let mut server = TestServer::start("127.0.0.1:9384").await;

    server
        .started_rx()
        .await
        .expect("Should be able to start server");

    let mut alice = Client::from_registration()
        .username("Alice")
        .device_name("Alice's Device")
        .store_config(SqliteStoreConfig::in_memory().await)
        .api_client_config(HttpClientConfig::new(address.clone()))
        .protocol_config(WebSocketProtocolClientConfig::new(address.clone()))
        .call()
        .await
        .unwrap();

    let publish_keys = alice
        .publish_prekeys()
        .onetime_prekeys(10)
        .new_signed_prekey(true)
        .new_last_resort(true)
        .call()
        .await;

    assert!(publish_keys.is_ok())
}

#[tokio::test]
pub async fn bob_can_fetch_alices_keys() {
    let _ = env_logger::try_init();
    let address = "127.0.0.1:9385".to_owned();
    let mut server = TestServer::start("127.0.0.1:9385").await;

    server
        .started_rx()
        .await
        .expect("Should be able to start server");

    let mut alice = Client::from_registration()
        .username("Alice")
        .device_name("Alice's Device")
        .store_config(SqliteStoreConfig::in_memory().await)
        .api_client_config(HttpClientConfig::new(address.clone()))
        .protocol_config(WebSocketProtocolClientConfig::new(address.clone()))
        .call()
        .await
        .unwrap();

    let mut bob = Client::from_registration()
        .username("Bob")
        .device_name("Bob's Device")
        .store_config(SqliteStoreConfig::in_memory().await)
        .api_client_config(HttpClientConfig::new(address.clone()))
        .protocol_config(WebSocketProtocolClientConfig::new(address))
        .call()
        .await
        .unwrap();

    alice
        .publish_prekeys()
        .onetime_prekeys(10)
        .new_signed_prekey(true)
        .new_last_resort(true)
        .call()
        .await
        .unwrap();

    let pre_key = bob
        .fetch_prekeys(alice.account_id().await.unwrap(), None)
        .await;

    assert!(pre_key.is_ok())
}

#[tokio::test]
pub async fn bob_can_fetch_alices_keys_for_specific_devices() {
    let _ = env_logger::try_init();
    let address = "127.0.0.1:9386".to_owned();
    let mut server = TestServer::start("127.0.0.1:9386").await;

    server
        .started_rx()
        .await
        .expect("Should be able to start server");

    let mut alice = Client::from_registration()
        .username("Alice")
        .device_name("Alice's Device")
        .store_config(SqliteStoreConfig::in_memory().await)
        .api_client_config(HttpClientConfig::new(address.clone()))
        .protocol_config(WebSocketProtocolClientConfig::new(address.clone()))
        .call()
        .await
        .unwrap();

    let mut bob = Client::from_registration()
        .username("Bob")
        .device_name("Bob's Device")
        .store_config(SqliteStoreConfig::in_memory().await)
        .api_client_config(HttpClientConfig::new(address.clone()))
        .protocol_config(WebSocketProtocolClientConfig::new(address))
        .call()
        .await
        .unwrap();

    alice
        .publish_prekeys()
        .onetime_prekeys(10)
        .new_signed_prekey(true)
        .new_last_resort(true)
        .call()
        .await
        .unwrap();

    let pre_key = bob
        .fetch_prekeys(alice.account_id().await.unwrap(), Some(vec![1.into()]))
        .await;

    assert!(pre_key.is_ok())
}
