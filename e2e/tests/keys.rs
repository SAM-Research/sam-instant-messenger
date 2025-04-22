use sam_client::client::SqliteClientType;
use sam_client::net::http_client::HttpClientConfig;
use sam_client::net::protocol::WebSocketProtocolClientConfig;
use sam_client::storage::sqlite::SqliteStoreConfig;
use sam_client::Client;
use test_utils::e2e::TestServer;
use test_utils::get_next_port;

#[tokio::test]
pub async fn alice_can_upload_keys() {
    let address = format!("127.0.0.1:{}", get_next_port());
    let mut server = TestServer::start(&address, None).await;

    server
        .started_rx()
        .await
        .expect("Should be able to start server");

    let mut alice: Client<SqliteClientType> = Client::from_registration()
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
