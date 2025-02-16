use libsignal_protocol::{
    GenericSignedPreKey, IdentityKeyPair, KeyPair, KyberPreKeyId, KyberPreKeyRecord, PreKeyId,
    PreKeyRecord, SignedPreKeyId, SignedPreKeyRecord, Timestamp,
};
use rand::rngs::OsRng;
use sam_common::{
    api::{
        device::DeviceActivationInfo, keys::PublishKeyBundle, EcPreKey, LinkDeviceRequest,
        LinkDeviceToken, PqPreKey, RegistrationRequest, SignedEcPreKey,
    },
    time_now_millis,
};

pub fn create_device_link(
    name: &str,
    reg_id: u32,
    pre_key_ids: Option<Vec<u32>>,
    signed_pre_key_id: Option<u32>,
    pq_pre_key_ids: Option<Vec<u32>>,
    pq_last_resort_pre_key_id: Option<u32>,
    token: LinkDeviceToken,
    pair: &IdentityKeyPair,
    rng: OsRng,
) -> LinkDeviceRequest {
    LinkDeviceRequest {
        token,
        device_activation: create_device_info(
            name,
            reg_id,
            pre_key_ids,
            signed_pre_key_id,
            pq_pre_key_ids,
            pq_last_resort_pre_key_id,
            pair,
            rng,
        ),
    }
}

pub fn create_publish_key_bundle(
    pre_key_ids: Option<Vec<u32>>,
    signed_pre_key_id: Option<u32>,
    pq_pre_key_ids: Option<Vec<u32>>,
    pq_last_resort_pre_key_id: Option<u32>,

    pair: &IdentityKeyPair,
    rng: OsRng,
) -> PublishKeyBundle {
    let pre_keys = pre_key_ids.and_then(|keys| {
        Some(
            keys.iter()
                .map(|id| ec_pre_key(*id, rng))
                .collect::<Vec<EcPreKey>>(),
        )
    });

    let signed_pre_key = signed_pre_key_id.and_then(|id| Some(signed_ec_pre_key(id, pair, rng)));

    let pq_pre_keys = pq_pre_key_ids.and_then(|keys| {
        Some(
            keys.iter()
                .map(|id| pq_pre_key(*id, pair))
                .collect::<Vec<PqPreKey>>(),
        )
    });
    let pq_last_resort_pre_key =
        pq_last_resort_pre_key_id.and_then(|id| Some(pq_pre_key(id, pair)));

    PublishKeyBundle {
        pre_keys,
        signed_pre_key,
        pq_pre_keys,
        pq_last_resort_pre_key,
    }
}

pub fn create_device_info(
    name: &str,
    reg_id: u32,
    pre_key_ids: Option<Vec<u32>>,
    signed_pre_key_id: Option<u32>,
    pq_pre_key_ids: Option<Vec<u32>>,
    pq_last_resort_pre_key_id: Option<u32>,
    pair: &IdentityKeyPair,
    rng: OsRng,
) -> DeviceActivationInfo {
    DeviceActivationInfo {
        name: name.to_string(),
        registration_id: reg_id,
        key_bundle: create_publish_key_bundle(
            pre_key_ids,
            signed_pre_key_id,
            pq_pre_key_ids,
            pq_last_resort_pre_key_id,
            pair,
            rng,
        ),
    }
}

pub fn create_registration(
    name: &str,
    reg_id: u32,
    pre_key_ids: Option<Vec<u32>>,
    signed_pre_key_id: Option<u32>,
    pq_pre_key_ids: Option<Vec<u32>>,
    pq_last_resort_pre_key_id: Option<u32>,
    pair: &IdentityKeyPair,
    rng: OsRng,
) -> RegistrationRequest {
    RegistrationRequest {
        identity_key: *pair.identity_key(),
        device_activation: create_device_info(
            name,
            reg_id,
            pre_key_ids,
            signed_pre_key_id,
            pq_pre_key_ids,
            pq_last_resort_pre_key_id,
            pair,
            rng,
        ),
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
