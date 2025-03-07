use std::time::Duration;

use criterion::{criterion_group, criterion_main, Criterion};
use sam_client::net::http_client::HttpClientConfig;
use sam_client::net::protocol::WebSocketProtocolClientConfig;
use sam_client::storage::sqlite::SqliteStoreConfig;
use sam_client::Client;
use sam_e2e_tests::TestServer;
use tokio::runtime::Runtime;

async fn register_client() {
    let address = "http://127.0.0.1:10000".to_owned();
    let mut server = TestServer::start("127.0.0.1:10000").await;

    server
        .started_rx()
        .await
        .expect("Should be able to start server");

    let _ = Client::from_registration()
        .username("Alice")
        .device_name("Alice's Device")
        .store_config(SqliteStoreConfig::in_memory().await)
        .api_client_config(HttpClientConfig::new(address.clone()))
        .protocol_config(WebSocketProtocolClientConfig::new(address.clone()))
        .call()
        .await;
}

fn from_elem(c: &mut Criterion) {
    c.bench_function("register_one_client", |b| {
        b.to_async(Runtime::new().expect("can create async runtime"))
            .iter(|| register_client());
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default().measurement_time(Duration::from_secs(8));
    targets = from_elem
}
criterion_main!(benches);
