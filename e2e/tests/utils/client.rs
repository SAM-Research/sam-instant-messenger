use libsignal_protocol::{
    GenericSignedPreKey, IdentityKey, IdentityKeyPair, KeyPair, KyberPreKeyRecord,
    SignedPreKeyRecord,
};
use rand::rngs::OsRng;
use sam_client::signal_time_now;
use sam_common::address::RegistrationId;
use sam_common::api::device::DeviceActivationInfo;
use sam_common::api::keys::RegistrationPreKeys;
use sam_common::api::{PublishPreKeys, RegistrationRequest};

pub fn registration_prekeys(id_key_pair: IdentityKeyPair) -> RegistrationPreKeys {
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

pub fn registration_request(id_key_pair: IdentityKeyPair) -> RegistrationRequest {
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

pub fn publish_keys(id_key_pair: IdentityKeyPair) -> PublishPreKeys {
    let keys = registration_prekeys(id_key_pair);
    PublishPreKeys {
        pre_keys: keys.pre_keys,
        signed_pre_key: Some(keys.signed_pre_key),
        pq_pre_keys: keys.pq_pre_keys,
        pq_last_resort_pre_key: Some(keys.pq_last_resort_pre_key),
    }
}
