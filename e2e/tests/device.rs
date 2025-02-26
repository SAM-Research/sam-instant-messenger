use libsignal_protocol::IdentityKeyPair;
use rand::rngs::OsRng;
use sam_client::net::ApiClient;
use sam_client::net::HttpClient;
use sam_common::address::RegistrationId;
use sam_common::api::device::DeviceActivationInfo;
use sam_common::api::keys::RegistrationPreKeys;
use sam_common::api::{LinkDeviceRequest, LinkDeviceToken};

use crate::utils::client::{publish_keys, registration_request};
use crate::utils::server::TestServer;

mod utils;

fn link_device_request(
    token: LinkDeviceToken,
    reg_id: RegistrationId,
    reg_keys: RegistrationPreKeys,
) -> LinkDeviceRequest {
    LinkDeviceRequest {
        token,
        device_activation: DeviceActivationInfo {
            name: "Secondary device".to_string(),
            registration_id: reg_id,
            key_bundle: reg_keys,
        },
    }
}

#[tokio::test]
async fn test_device_provision() {
    let _ = env_logger::try_init();
    let address = "http://127.0.0.1:9387";
    let mut server = TestServer::start("127.0.0.1:9387").await;
    let mut csprng = OsRng;
    let id_key_pair = IdentityKeyPair::generate(&mut csprng);

    server
        .started_rx()
        .await
        .expect("Should be able to start server");

    let client = HttpClient::new(address.to_owned());

    let username = "Alice";
    let password = "Alice Password";

    let result = client
        .register_account(username, password, registration_request(id_key_pair))
        .await;

    assert!(result.is_ok());

    let account_id = result.unwrap().account_id;

    let token_result = client
        .provision_device(account_id, 1.into(), password)
        .await;

    assert!(token_result.is_ok());
}

#[tokio::test]
async fn test_link_device() {
    let _ = env_logger::try_init();
    let address = "http://127.0.0.1:9388";
    let mut server = TestServer::start("127.0.0.1:9388").await;
    let mut csprng = OsRng;
    let id_key_pair = IdentityKeyPair::generate(&mut csprng);

    server
        .started_rx()
        .await
        .expect("Should be able to start server");

    let client = HttpClient::new(address.to_owned());

    let username = "Alice";
    let password = "Alice Password";

    let result = client
        .register_account(username, password, registration_request(id_key_pair))
        .await;

    assert!(result.is_ok());

    let account_id = result.unwrap().account_id;

    let token_result = client
        .provision_device(account_id, 1.into(), password)
        .await;

    assert!(token_result.is_ok());

    let token = token_result.unwrap();
    let registration_pre_keys = publish_keys(id_key_pair)
        .try_into()
        .expect("Can make RegistrationPreKeys");

    assert!(client
        .link_device(
            account_id,
            1.into(),
            password,
            link_device_request(token, 2.into(), registration_pre_keys)
        )
        .await
        .is_ok());
}

#[tokio::test]
async fn test_delete_device() {
    let _ = env_logger::try_init();
    let address = "http://127.0.0.1:9389";
    let mut server = TestServer::start("127.0.0.1:9389").await;
    let mut csprng = OsRng;
    let id_key_pair = IdentityKeyPair::generate(&mut csprng);

    server
        .started_rx()
        .await
        .expect("Should be able to start server");

    let client = HttpClient::new(address.to_owned());

    let username = "Alice";
    let password = "Alice Password";

    let result = client
        .register_account(username, password, registration_request(id_key_pair))
        .await;

    assert!(result.is_ok());

    let account_id = result.unwrap().account_id;

    let token_result = client
        .provision_device(account_id, 1.into(), password)
        .await;

    assert!(token_result.is_ok());

    let token = token_result.unwrap();
    let registration_pre_keys = publish_keys(id_key_pair)
        .try_into()
        .expect("Can make RegistrationPreKeys");

    let link_device_reponse = client
        .link_device(
            account_id,
            1.into(),
            password,
            link_device_request(token, 2.into(), registration_pre_keys),
        )
        .await;

    assert!(link_device_reponse.is_ok());

    assert!(client
        .delete_device(account_id, 1.into(), password, 2.into())
        .await
        .is_ok())
}
