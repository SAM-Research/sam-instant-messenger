use utils::TestServer;
mod utils;

#[tokio::test]
pub async fn one_client_can_register() {
    let address = "127.0.0.1:9384";
    let mut server = TestServer::start(address).await;

    server
        .started_rx()
        .await
        .expect("Should be able to start server");

    /*
    let storage_config = SqliteStoreConfig::in_memory().await;

    assert!(Client::register(
        "Alice",
        "Alice's Device",
        storage_config,
        HttpClient::new(address.to_owned())
    )
    .await
    .is_ok());
    */
}
