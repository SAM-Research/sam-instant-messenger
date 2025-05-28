use std::time::Duration;

use env_logger::fmt::Formatter;
use log::info;
use sam_client::{
    client::{ClientType, SqliteClientType},
    encryption::DecryptedEnvelope,
    net::{protocol::WebSocketProtocolClientConfig, HttpClientConfig},
    storage::SqliteStoreConfig,
    Client,
};
use sam_common::{time_now_millis, AccountId};
use sam_net::tls::{create_tls_client_config, MutualTlsConfig};
use sam_server::config::TlsConfig;
use sam_test_utils::e2e::{in_memory_server_state, TestServer};
use std::io::Write;
use tokio::sync::broadcast::Receiver;
use uuid::Uuid;

async fn tls_client(
    address: &str,
    device_name: &str,
    mutual_config: Option<MutualTlsConfig>,
) -> Client<SqliteClientType> {
    // wireshark filter: tcp.port == 9443
    let client_config = create_tls_client_config("./cert/rootCA.crt", mutual_config)
        .expect("Can create client config");
    let username = Uuid::new_v4().to_string();
    Client::from_registration()
        .username(&username)
        .device_name(device_name)
        .store_config(
            SqliteStoreConfig::in_memory(10)
                .await
                .expect("can create inmemory"),
        )
        .api_client_config(HttpClientConfig::new_with_tls(
            address.to_string(),
            client_config.clone(),
        ))
        .protocol_config(WebSocketProtocolClientConfig::new_with_tls(
            address.to_string(),
            client_config,
            10,
        ))
        .upload_prekey_count(5)
        .call()
        .await
        .expect("Can register Client")
}

#[tokio::main]
async fn main() {
    let millis = 5000;
    env_logger::builder()
        .format(|buf: &mut Formatter, record: &log::Record| {
            let now = time_now_millis();

            writeln!(buf, "{} |{}| {}", now, record.target(), record.args())
        })
        .parse_filters("sam_server=info,sam_client=info,sam_e2e_tests=info")
        .init();
    let _ = rustls::crypto::ring::default_provider().install_default();
    let mutual_config = Some(MutualTlsConfig::new(
        "./cert/client.key".to_string(),
        "./cert/client.crt".to_string(),
    ));
    let address = format!("127.0.0.1:{}", "9443");
    let server_config = TlsConfig {
        ca_cert_path: Some("./cert/rootCA.crt".to_string()),
        cert_path: "./cert/server.crt".to_string(),
        key_path: "./cert/server.key".to_string(),
    }
    .try_into()
    .expect("Can create server config");
    let mut server = TestServer::start(
        &address,
        Some(server_config),
        in_memory_server_state().await,
    )
    .await;
    server
        .started_rx()
        .await
        .expect("Should be able to start server");
    let mut alice = tls_client(&address, "alice device", mutual_config.clone()).await;
    let mut bob = tls_client(&address, "bob device", mutual_config.clone()).await;
    let mut charlie = tls_client(&address, "charlie device", mutual_config.clone()).await;
    let mut dorothy = tls_client(&address, "dorothy device", mutual_config.clone()).await;

    let mut a_rx = alice.subscribe();
    let mut b_rx = bob.subscribe();
    let mut c_rx = charlie.subscribe();
    let mut d_rx = dorothy.subscribe();

    let alice_id = alice.account_id();
    let bob_id = bob.account_id();
    let charlie_id = charlie.account_id();
    let dorothy_id = dorothy.account_id();

    ////////////////////////////////////////////////////////
    let a_msg = [8u8; 400];
    let b_msg = [16u8; 450];
    let c_msg = [32u8; 500];
    let d_msg = [64u8; 550];

    // ##### Expirment #####
    info!("Alice {alice_id}");
    info!("Alice msg {}", a_msg.len());
    info!("---------");
    info!("Bob {bob_id}");
    info!("Bob msg {}", b_msg.len());
    info!("---------");
    info!("Charlie {charlie_id}");
    info!("Charlie msg {}", c_msg.len());
    info!("---------");
    info!("Dorothy {dorothy_id}");
    info!("Dorothy msg {}", d_msg.len());

    // key uploads
    tokio::time::sleep(Duration::from_millis(millis)).await;
    info!("\n\n-------------------- SEED UPDATES ------------------");
    send_recv(&mut alice, &mut bob, &mut b_rx, a_msg).await;
    send_recv(&mut bob, &mut alice, &mut a_rx, b_msg).await;
    send_recv(&mut charlie, &mut dorothy, &mut d_rx, c_msg).await;
    send_recv(&mut dorothy, &mut charlie, &mut c_rx, d_msg).await;

    // key request + inital deniable message
    tokio::time::sleep(Duration::from_millis(millis)).await;
    info!("\n\n------------- ALICE KEY REQ + ENQUEUE DENIM --------");
    send_recv(&mut alice, &mut bob, &mut b_rx, a_msg).await;

    // key response
    tokio::time::sleep(Duration::from_millis(millis)).await;
    info!("\n\n--------------- KEY RESPONSE ----------------------");
    send_recv(&mut bob, &mut alice, &mut a_rx, b_msg).await;

    // piggy back denim message
    tokio::time::sleep(Duration::from_millis(millis)).await;
    info!("\n\n--------------- SEND DENIM ------------------------");
    send_recv(&mut alice, &mut dorothy, &mut d_rx, a_msg).await;
    send_recv(&mut alice, &mut bob, &mut b_rx, a_msg).await;

    // dorothy receives denim message
    tokio::time::sleep(Duration::from_millis(millis)).await;
    info!("\n\n---------------- RECEIVE DENIM --------------------");
    send_recv(&mut charlie, &mut dorothy, &mut d_rx, c_msg).await;

    // piggy back denim message
    tokio::time::sleep(Duration::from_millis(millis)).await;
    info!("\n\n--------------- SEND DENIM ------------------------");

    send_recv(&mut dorothy, &mut alice, &mut a_rx, d_msg).await;
    send_recv(&mut dorothy, &mut charlie, &mut c_rx, d_msg).await;

    // alice receives dorothy denim message
    tokio::time::sleep(Duration::from_millis(millis)).await;
    info!("\n\n---------------- RECEIVE DENIM --------------------");
    send_recv(&mut bob, &mut alice, &mut a_rx, b_msg).await;
}

async fn send_recv(
    a: &mut Client<impl ClientType>,
    b: &mut Client<impl ClientType>,
    b_rx: &mut Receiver<DecryptedEnvelope>,
    a_msg: impl Into<Vec<u8>> + Clone,
) {
    let bid = b.account_id();
    a.send_message(bid, a_msg).await.expect("can send message");

    b.process_messages_blocking().await.expect("can process");

    let env = b_rx.recv().await.expect("can recv");
    log_recv(bid, env, false);
}

fn log_recv(me: AccountId, env: DecryptedEnvelope, denim: bool) {
    let sender = env.source_account_id();
    let len = env.content_bytes().len();
    let x = if denim { "DENIM " } else { "" };
    info!("{me} <-({len})- {sender} {x}");
}
