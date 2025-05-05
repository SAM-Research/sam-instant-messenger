use rstest::rstest;
use sam_client::client::SqliteClientType;
use sam_client::net::http_client::HttpClientConfig;
use sam_client::net::protocol::WebSocketProtocolClientConfig;
use sam_client::storage::sqlite::SqliteStoreConfig;
use sam_client::Client;
use sam_server::{ServerState, StateType};
use sam_test_utils::{
    e2e::{in_memory_server_state, postgres_server_state, TestServer},
    get_next_port,
};
use uuid::Uuid;

#[tokio::test]
#[rstest]
#[ignore = "requires a postgres test database"]
#[case::postgres(postgres_server_state())]
#[case::in_memory(in_memory_server_state())]
pub async fn alice_can_upload_keys(
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

    let mut alice: Client<SqliteClientType> = Client::from_registration()
        .username(&Uuid::new_v4().to_string())
        .device_name("Alice's Device")
        .store_config(
            SqliteStoreConfig::in_memory(10)
                .await
                .expect("can create inmemory"),
        )
        .api_client_config(HttpClientConfig::new(address.clone()))
        .protocol_config(WebSocketProtocolClientConfig::new(address.clone(), 10))
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
