use crate::utils::server::TestServer;
use sam_client::client::SqliteClientType;
use sam_client::net::http_client::HttpClientConfig;
use sam_client::net::protocol::WebSocketProtocolClientConfig;
use sam_client::storage::sqlite::{SqliteSamStoreConfig, SqliteSignalStoreConfig};
use sam_client::Client;

mod utils;

/*
   PORTS USED: 938x
*/

#[tokio::test]
async fn can_link_device() {
    let address = "127.0.0.1:9380";
    let mut server = TestServer::start(address, None).await;

    server
        .started_rx()
        .await
        .expect("Should be able to start server");

    let username = "Alice";
    let device_name = "Alice's Device";

    let signal_store_config = SqliteSignalStoreConfig::in_memory().await;
    let sam_store_config = SqliteSamStoreConfig::in_memory().await;
    let protocol_config = WebSocketProtocolClientConfig::new(address.to_owned());
    let api_client_config = HttpClientConfig::new(address.to_owned());

    let mut alice: Client<SqliteClientType> = Client::from_registration()
        .signal_store_config(signal_store_config)
        .sam_store_config(sam_store_config)
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

    let signal_store_config = SqliteSignalStoreConfig::in_memory().await;
    let sam_store_config = SqliteSamStoreConfig::in_memory().await;
    let protocol_config = WebSocketProtocolClientConfig::new(address.to_owned());
    let api_client_config = HttpClientConfig::new(address.to_owned());
    let id_key_pair = alice
        .identity_key_pair()
        .await
        .expect("Can get id key pair");

    assert!(Client::<SqliteClientType>::from_provisioning()
        .protocol_config(protocol_config)
        .api_client_config(api_client_config)
        .signal_store_config(signal_store_config)
        .sam_store_config(sam_store_config)
        .device_name("Alice's Other Device")
        .id_key_pair(id_key_pair)
        .token(token)
        .call()
        .await
        .is_ok());
}

#[tokio::test]
async fn can_unlink_device() {
    let address = "127.0.0.1:9381";
    let mut server = TestServer::start(address, None).await;

    server
        .started_rx()
        .await
        .expect("Should be able to start server");

    let username = "Alice";
    let device_name = "Alice's Device";

    let signal_store_config = SqliteSignalStoreConfig::in_memory().await;
    let sam_store_config = SqliteSamStoreConfig::in_memory().await;
    let protocol_config = WebSocketProtocolClientConfig::new(address.to_owned());
    let api_client_config = HttpClientConfig::new(address.to_owned());

    let mut alice: Client<SqliteClientType> = Client::from_registration()
        .signal_store_config(signal_store_config)
        .sam_store_config(sam_store_config)
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

    let signal_store_config = SqliteSignalStoreConfig::in_memory().await;
    let sam_store_config = SqliteSamStoreConfig::in_memory().await;
    let protocol_config = WebSocketProtocolClientConfig::new(address.to_owned());
    let api_client_config = HttpClientConfig::new(address.to_owned());
    let id_key_pair = alice
        .identity_key_pair()
        .await
        .expect("Can get id key pair");

    let other_client: Client<SqliteClientType> = Client::from_provisioning()
        .protocol_config(protocol_config)
        .api_client_config(api_client_config)
        .signal_store_config(signal_store_config)
        .sam_store_config(sam_store_config)
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
async fn can_delete_device() {
    let address = "127.0.0.1:9382";
    let mut server = TestServer::start(address, None).await;

    server
        .started_rx()
        .await
        .expect("Should be able to start server");

    let username = "Alice";
    let device_name = "Alice's Device";

    let signal_store_config = SqliteSignalStoreConfig::in_memory().await;
    let sam_store_config = SqliteSamStoreConfig::in_memory().await;
    let protocol_config = WebSocketProtocolClientConfig::new(address.to_owned());
    let api_client_config = HttpClientConfig::new(address.to_owned());

    let mut alice: Client<SqliteClientType> = Client::from_registration()
        .signal_store_config(signal_store_config)
        .sam_store_config(sam_store_config)
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

    let signal_store_config = SqliteSignalStoreConfig::in_memory().await;
    let sam_store_config = SqliteSamStoreConfig::in_memory().await;
    let protocol_config = WebSocketProtocolClientConfig::new(address.to_owned());
    let api_client_config = HttpClientConfig::new(address.to_owned());
    let id_key_pair = alice
        .identity_key_pair()
        .await
        .expect("Can get id key pair");

    let other_client: Client<SqliteClientType> = Client::from_provisioning()
        .protocol_config(protocol_config)
        .api_client_config(api_client_config)
        .signal_store_config(signal_store_config)
        .sam_store_config(sam_store_config)
        .device_name("Alice's Other Device")
        .id_key_pair(id_key_pair)
        .token(token)
        .call()
        .await
        .expect("Can link device");

    assert!(other_client.delete_device().await.is_ok())
}

#[tokio::test]
async fn can_delete_account() {
    let address = "127.0.0.1:9383";
    let mut server = TestServer::start(address, None).await;

    server
        .started_rx()
        .await
        .expect("Should be able to start server");

    let username = "Alice";
    let device_name = "Alice's Device";

    let signal_store_config = SqliteSignalStoreConfig::in_memory().await;
    let sam_store_config = SqliteSamStoreConfig::in_memory().await;
    let protocol_config = WebSocketProtocolClientConfig::new(address.to_owned());
    let api_client_config = HttpClientConfig::new(address.to_owned());

    let alice: Client<SqliteClientType> = Client::from_registration()
        .signal_store_config(signal_store_config)
        .sam_store_config(sam_store_config)
        .protocol_config(protocol_config)
        .api_client_config(api_client_config)
        .username(username)
        .device_name(device_name)
        .call()
        .await
        .expect("Can register account");

    assert!(alice.delete_account().await.is_ok())
}
