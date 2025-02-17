use libsignal_protocol::{
    GenericSignedPreKey, IdentityKeyPair, KeyPair, KyberPreKeyId, KyberPreKeyRecord, PreKeyId,
    PreKeyRecord, SignedPreKeyId, SignedPreKeyRecord, Timestamp,
};
use rand::rngs::OsRng;
use sam_common::{
    api::{
        device::DeviceActivationInfo, keys::PublishPreKeys, EcPreKey, LinkDeviceRequest,
        LinkDeviceToken, PqPreKey, SignedEcPreKey,
    },
    time_now_millis,
};

pub fn create_device_link(
    token: LinkDeviceToken,
    name: &str,
    registration_id: u32,
    key_bundle: PublishPreKeys,
) -> LinkDeviceRequest {
    LinkDeviceRequest {
        token,
        device_activation: DeviceActivationInfo {
            name: name.to_string(),
            registration_id,
            key_bundle,
        },
    }
}

pub fn create_publish_key_bundle(
    pre_key_ids: Option<Vec<u32>>,
    signed_pre_key_id: Option<u32>,
    pq_pre_key_ids: Option<Vec<u32>>,
    pq_last_resort_pre_key_id: Option<u32>,

    pair: &IdentityKeyPair,
    rng: OsRng,
) -> PublishPreKeys {
    let pre_keys = pre_key_ids.map(|keys| {
        keys.iter()
            .map(|id| ec_pre_key(*id, rng))
            .collect::<Vec<EcPreKey>>()
    });

    let signed_pre_key = signed_pre_key_id.map(|id| signed_ec_pre_key(id, pair, rng));

    let pq_pre_keys = pq_pre_key_ids.map(|keys| {
        keys.iter()
            .map(|id| pq_pre_key(*id, pair))
            .collect::<Vec<PqPreKey>>()
    });
    let pq_last_resort_pre_key = pq_last_resort_pre_key_id.map(|id| pq_pre_key(id, pair));

    PublishPreKeys {
        pre_keys,
        signed_pre_key,
        pq_pre_keys,
        pq_last_resort_pre_key,
    }
}

pub fn ec_pre_key(id: u32, mut rng: OsRng) -> EcPreKey {
    PreKeyRecord::new(PreKeyId::from(id), &KeyPair::generate(&mut rng)).into()
}

pub fn signed_ec_pre_key(id: u32, pair: &IdentityKeyPair, mut rng: OsRng) -> SignedEcPreKey {
    let id = SignedPreKeyId::from(id);
    let pre_key = KeyPair::generate(&mut rng);
    let signature = pair
        .private_key()
        .calculate_signature(&pre_key.public_key.serialize(), &mut rng)
        .expect("Signal Signature works");
    SignedPreKeyRecord::new(
        id,
        Timestamp::from_epoch_millis(time_now_millis().try_into().expect("Time works")),
        &pre_key,
        &signature,
    )
    .into()
}

pub fn pq_pre_key(id: u32, pair: &IdentityKeyPair) -> PqPreKey {
    KyberPreKeyRecord::generate(
        libsignal_protocol::kem::KeyType::Kyber1024,
        KyberPreKeyId::from(id),
        pair.private_key(),
    )
    .expect("Keys can be generated")
    .into()
}
