use super::{signal_in_mem, signal_sqlite};
use libsignal_protocol::IdentityKeyStore as _;
use libsignal_protocol::{
    GenericSignedPreKey as _, IdentityKeyPair, KeyPair, SignedPreKeyRecord, SignedPreKeyStore,
};
use rand::rngs::OsRng;
use rand::{rngs::StdRng, SeedableRng as _};
use rstest::rstest;
use sam_client::storage::key_generation::SignedPreKeyGenerator as _;
use sam_client::storage::{SignalStore, SignalStoreType};
use sam_client::{signal_time_now, storage::SamStore};

#[rstest]
#[case(signal_in_mem().await.signed_pre_key_store)]
#[case(signal_sqlite().await.signed_pre_key_store)]
#[tokio::test]
async fn signed_pre_key_can_be_saved_and_retrieved(
    #[case] mut signed_pre_key_store: impl SignedPreKeyStore,
) {
    let mut csprng = OsRng;
    let identity_key = IdentityKeyPair::generate(&mut csprng);

    let signed_pre_key_pair = KeyPair::generate(&mut csprng);
    let signature = identity_key
        .private_key()
        .calculate_signature(&signed_pre_key_pair.public_key.serialize(), &mut csprng)
        .expect("should be able to sign pre key record");

    let signed_pre_key_record = SignedPreKeyRecord::new(
        0.into(),
        signal_time_now(),
        &signed_pre_key_pair,
        &signature,
    );

    signed_pre_key_store
        .save_signed_pre_key(signed_pre_key_record.id().unwrap(), &signed_pre_key_record)
        .await
        .unwrap();

    let retrived_record = signed_pre_key_store
        .get_signed_pre_key(signed_pre_key_record.id().unwrap())
        .await
        .unwrap();

    assert_eq!(
        retrived_record.id().unwrap(),
        signed_pre_key_record.id().unwrap()
    );
    assert_eq!(
        retrived_record.public_key().unwrap(),
        signed_pre_key_record.key_pair().unwrap().public_key
    );
    assert_eq!(
        retrived_record.private_key().unwrap().serialize(),
        signed_pre_key_record
            .key_pair()
            .unwrap()
            .private_key
            .serialize()
    );
}

#[rstest]
#[case(signal_in_mem().await)]
#[case(signal_sqlite().await)]
#[tokio::test]
async fn signed_pre_keys_ids_are_generated_properly(
    #[case] mut store: SignalStore<impl SignalStoreType>,
) {
    let mut rng = StdRng::seed_from_u64(128);
    let expected: Vec<u32> = (1u32..=10u32).collect();

    let mut ids: Vec<u32> = Vec::new();
    let id_key_pair = store
        .identity_key_store
        .get_identity_key_pair()
        .await
        .expect("Can get id key pair");

    for _ in 1u32..=10u32 {
        ids.push(
            store
                .signed_pre_key_store
                .generate_key(&mut rng, id_key_pair.private_key())
                .await
                .expect("Can generate keys")
                .id()
                .expect("Can get id of key")
                .into(),
        );
    }

    assert_eq!(expected, ids)
}
