use libsignal_protocol::IdentityKeyPair;
use rand::rngs::OsRng;
use sam_client::net::api_trait::SamApiClient;
use sam_client::net::http_client::HttpClient;

mod utils;

use crate::utils::client::{publish_keys, registration_request};
use crate::utils::server::TestServer;

#[tokio::test]
pub async fn bob_can_fetch_alices_keys() {
    let _ = env_logger::try_init();
    let address = "http://127.0.0.1:9385";
    let mut server = TestServer::start("127.0.0.1:9385").await;
    let alice_password = "Alice Password";
    let mut csprng = OsRng;
    let alice_id_key_pair = IdentityKeyPair::generate(&mut csprng);

    server
        .started_rx()
        .await
        .expect("Should be able to start server");

    let alice_client = HttpClient::new(address.to_owned());

    let alice_account_creation_result = alice_client
        .register_account(
            "Alice",
            alice_password,
            registration_request(alice_id_key_pair),
        )
        .await;

    assert!(alice_account_creation_result.is_ok());

    let alice_account_id = alice_account_creation_result.unwrap().account_id;

    assert!(alice_client
        .publish_pre_keys(
            &alice_account_id,
            1.into(),
            alice_password,
            publish_keys(alice_id_key_pair)
        )
        .await
        .inspect_err(|err| println!("{err}"))
        .is_ok());

    // now we create Bob

    let bob_client = HttpClient::new(address.to_owned());
    let bob_id_key_pair = IdentityKeyPair::generate(&mut csprng);
    let bob_password = "Bob password";

    let bob_account_creation_result = bob_client
        .register_account("Bob", bob_password, registration_request(bob_id_key_pair))
        .await;

    let bob_account_id = bob_account_creation_result.unwrap().account_id;

    let get_prekeys_result = bob_client
        .get_pre_keys(&bob_account_id, 1.into(), bob_password, alice_account_id)
        .await;

    assert!(get_prekeys_result.is_ok())
}
