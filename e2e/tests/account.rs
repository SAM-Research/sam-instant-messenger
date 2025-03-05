use crate::utils::client::registration_request;
use crate::utils::server::TestServer;
use libsignal_protocol::IdentityKeyPair;
use rand::rngs::OsRng;
use sam_client::net::{ApiClient, HttpClient};
mod utils;

#[tokio::test]
pub async fn alice_can_find_bobs_account_id() {
    let _ = env_logger::try_init();
    let address = "http://127.0.0.1:9384";
    let mut server = TestServer::start("127.0.0.1:9384").await;
    let mut csprng = OsRng;
    let alice_id_key_pair = IdentityKeyPair::generate(&mut csprng);

    server
        .started_rx()
        .await
        .expect("Should be able to start server");

    let alice = HttpClient::new(address.to_owned());
    let alice_password = "Alice Password";

    let alice_result = alice
        .register_account(
            "Alice",
            alice_password,
            registration_request(alice_id_key_pair),
        )
        .await
        .unwrap();

    let alice_account_id = alice_result.account_id;

    let bob_id_key_pair = IdentityKeyPair::generate(&mut csprng);
    let bob = HttpClient::new(address.to_owned());
    let bob_result = bob
        .register_account("Bob", "Bob Password", registration_request(bob_id_key_pair))
        .await
        .unwrap();

    let result = alice
        .get_user_account_id(alice_account_id, 1.into(), alice_password, "Bob")
        .await;

    assert!(result.is_ok());
    assert_eq!(bob_result.account_id, result.unwrap())
}
