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
    api::{device::DeviceActivationInfo, keys::RegistrationPreKeys, RegistrationRequest},
};
use utils::TestServer;
mod utils;

#[tokio::test]
pub async fn one_client_can_register() {
    env_logger::init();
    let address = "127.0.0.1:9384";
    let mut server = TestServer::start(address).await;

    server
        .started_rx()
        .await
        .expect("Should be able to start server");

    let client = HttpClient::new(address.to_owned());

    let mut csprng = OsRng;
    let id_key_pair = IdentityKeyPair::generate(&mut csprng);

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

    let keys = RegistrationPreKeys {
        pre_keys: None,
        signed_pre_key: signed_pre_key.into(),
        pq_pre_keys: None,
        pq_last_resort_pre_key: last_resort_pq.into(),
    };
    let request = RegistrationRequest {
        identity_key: IdentityKey::new(id_key_pair.public_key().to_owned()),
        device_activation: DeviceActivationInfo {
            name: "Alice Device".to_owned(),
            registration_id: RegistrationId::generate(&mut csprng),
            key_bundle: keys,
        },
    };

    let result = client
        .register_account("Alice", "Alice Password", request)
        .await;

    assert!(result.is_ok());
}
