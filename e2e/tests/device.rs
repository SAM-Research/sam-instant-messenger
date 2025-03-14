use crate::utils::server::TestServer;
use sam_client::net::http_client::HttpClientConfig;
use sam_client::net::protocol::WebSocketProtocolClientConfig;
use sam_client::storage::sqlite::SqliteStoreConfig;
use sam_client::Client;

mod utils;

/*
   PORTS USED: 9388-9390
*/

#[tokio::test]
async fn can_link_device() {
    let _ = env_logger::try_init();
    let address = "127.0.0.1:9388";
    let mut server = TestServer::start(address, None).await;

    server
        .started_rx()
        .await
        .expect("Should be able to start server");

    let username = "Alice";
    let device_name = "Alice's Device";

    let store_config = SqliteStoreConfig::in_memory().await;
    let protocol_config = WebSocketProtocolClientConfig::new(address.to_owned(), None);
    let api_client_config = HttpClientConfig::new(address.to_owned(), None);

    let mut alice = Client::from_registration()
        .store_config(store_config)
        .protocol_config(protocol_config)
        .api_client_config(api_client_config)
        .username(username)
        .device_name(device_name)
        .call()
        .await
        .expect("Can register account");

    let token = alice
        .create_provision()
        .await
        .expect("Can create a link token");

    let store_config = SqliteStoreConfig::in_memory().await;
    let protocol_config = WebSocketProtocolClientConfig::new(address.to_owned(), None);
    let api_client_config = HttpClientConfig::new(address.to_owned(), None);
    let id_key_pair = alice
        .identity_key_pair()
        .await
        .expect("Can get id key pair");

    assert!(Client::from_provisioning()
        .protocol_config(protocol_config)
        .api_client_config(api_client_config)
        .store_config(store_config)
        .device_name("Alice's Other Device")
        .id_key_pair(id_key_pair)
        .token(token)
        .call()
        .await
        .is_ok());
}

#[tokio::test]
async fn can_delete_device() {
    let _ = env_logger::try_init();
    let address = "127.0.0.1:9389";
    let mut server = TestServer::start(address, None).await;

    server
        .started_rx()
        .await
        .expect("Should be able to start server");

    let username = "Alice";
    let device_name = "Alice's Device";

    let store_config = SqliteStoreConfig::in_memory().await;
    let protocol_config = WebSocketProtocolClientConfig::new(address.to_owned(), None);
    let api_client_config = HttpClientConfig::new(address.to_owned(), None);

    let mut alice = Client::from_registration()
        .store_config(store_config)
        .protocol_config(protocol_config)
        .api_client_config(api_client_config)
        .username(username)
        .device_name(device_name)
        .call()
        .await
        .expect("Can register account");

    let token = alice
        .create_provision()
        .await
        .expect("Can create a link token");

    let store_config = SqliteStoreConfig::in_memory().await;
    let protocol_config = WebSocketProtocolClientConfig::new(address.to_owned(), None);
    let api_client_config = HttpClientConfig::new(address.to_owned(), None);
    let id_key_pair = alice
        .identity_key_pair()
        .await
        .expect("Can get id key pair");

    let other_client = Client::from_provisioning()
        .protocol_config(protocol_config)
        .api_client_config(api_client_config)
        .store_config(store_config)
        .device_name("Alice's Other Device")
        .id_key_pair(id_key_pair)
        .token(token)
        .call()
        .await
        .expect("Can link device");

    assert!(alice
        .unlink_device(other_client.device_id().await.expect("Can get device_id"))
        .await
        .is_ok())
}

#[tokio::test]
async fn can_delete_account() {
    let _ = env_logger::try_init();
    let address = "127.0.0.1:9390";
    let mut server = TestServer::start(address, None).await;

    server
        .started_rx()
        .await
        .expect("Should be able to start server");

    let username = "Alice";
    let device_name = "Alice's Device";

    let store_config = SqliteStoreConfig::in_memory().await;
    let protocol_config = WebSocketProtocolClientConfig::new(address.to_owned(), None);
    let api_client_config = HttpClientConfig::new(address.to_owned(), None);

    let alice = Client::from_registration()
        .store_config(store_config)
        .protocol_config(protocol_config)
        .api_client_config(api_client_config)
        .username(username)
        .device_name(device_name)
        .call()
        .await
        .expect("Can register account");

    assert!(alice.delete_account().await.is_ok())
}
