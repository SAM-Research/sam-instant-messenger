use libsignal_protocol::{
    GenericSignedPreKey, IdentityKey, IdentityKeyPair, KeyPair, KyberPreKeyRecord,
    SignedPreKeyRecord,
};
use rand::rngs::OsRng;
use sam_client::{
    net::{api_trait::SamApiClient, http_client::HttpClient},
    signal_time_now,
};
use sam_common::{
    address::RegistrationId,
    api::{
        device::DeviceActivationInfo, keys::RegistrationPreKeys, PublishPreKeys,
        RegistrationRequest,
    },
};
use utils::TestServer;
mod utils;

fn registration_request(id_key_pair: IdentityKeyPair) -> RegistrationRequest {
    let mut csprng = OsRng;

    let keys = registration_prekeys(id_key_pair);
    RegistrationRequest {
        identity_key: IdentityKey::new(id_key_pair.public_key().to_owned()),
        device_activation: DeviceActivationInfo {
            name: "Alice Device".to_owned(),
            registration_id: RegistrationId::generate(&mut csprng),
            key_bundle: keys,
        },
    }
}

fn registration_prekeys(id_key_pair: IdentityKeyPair) -> RegistrationPreKeys {
    let mut csprng = OsRng;
    let last_resort_pq = KyberPreKeyRecord::generate(
        libsignal_protocol::kem::KeyType::Kyber1024,
        0.into(),
        id_key_pair.private_key(),
    )
    .unwrap();

    let key = KeyPair::generate(&mut csprng);
    let signed_pre_key = SignedPreKeyRecord::new(
        0.into(),
        signal_time_now(),
        &key,
        &id_key_pair
            .private_key()
            .calculate_signature(&key.public_key.serialize(), &mut csprng)
            .unwrap(),
    );

    RegistrationPreKeys {
        pre_keys: None,
        signed_pre_key: signed_pre_key.into(),
        pq_pre_keys: None,
        pq_last_resort_pre_key: last_resort_pq.into(),
    }
}

fn publish_keys(id_key_pair: IdentityKeyPair) -> PublishPreKeys {
    let keys = registration_prekeys(id_key_pair);
    PublishPreKeys {
        pre_keys: keys.pre_keys,
        signed_pre_key: Some(keys.signed_pre_key),
        pq_pre_keys: keys.pq_pre_keys,
        pq_last_resort_pre_key: Some(keys.pq_last_resort_pre_key),
    }
}

#[tokio::test]
pub async fn one_client_can_register() {
    let _ = env_logger::try_init();
    let address = "127.0.0.1:9384";
    let mut server = TestServer::start(address).await;
    let mut csprng = OsRng;
    let id_key_pair = IdentityKeyPair::generate(&mut csprng);

    server
        .started_rx()
        .await
        .expect("Should be able to start server");

    let client = HttpClient::new(address.to_owned());

    let result = client
        .register_account("Alice", "Alice Password", registration_request(id_key_pair))
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
pub async fn can_upload_keys() {
    let _ = env_logger::try_init();
    let address = "127.0.0.1:9384";
    let mut server = TestServer::start(address).await;
    let password = "Alice Password";
    let mut csprng = OsRng;
    let id_key_pair = IdentityKeyPair::generate(&mut csprng);

    server
        .started_rx()
        .await
        .expect("Should be able to start server");

    let client = HttpClient::new(address.to_owned());

    let result = client
        .register_account("Alice", password, registration_request(id_key_pair))
        .await;

    assert!(result.is_ok());

    let account_id = result.unwrap().account_id;
    let username = format!("{account_id}.1");

    assert!(client
        .publish_pre_keys(&username, password, publish_keys(id_key_pair))
        .await
        .inspect_err(|err| println!("{err}"))
        .is_ok())
}

#[tokio::test]
pub async fn can_can_delete_account() {
    let _ = env_logger::try_init();
    let address = "127.0.0.1:9384";
    let mut server = TestServer::start(address).await;
    let mut csprng = OsRng;
    let id_key_pair = IdentityKeyPair::generate(&mut csprng);

    server
        .started_rx()
        .await
        .expect("Should be able to start server");

    let client = HttpClient::new(address.to_owned());

    let result = client
        .register_account("Alice", "Alice Password", registration_request(id_key_pair))
        .await;

    assert!(result.is_ok());

    let account_id = result.unwrap().account_id;

    assert!(client
        .delete_account(&format!("{account_id}.1"), "Alice Password")
        .await
        .inspect_err(|err| println!("{err}"))
        .is_ok());
}
