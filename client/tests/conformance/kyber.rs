use super::{signal_in_mem, signal_sqlite};
use libsignal_protocol::{
    kem::KeyType, GenericSignedPreKey as _, IdentityKeyPair, IdentityKeyStore as _,
    KyberPreKeyRecord, KyberPreKeyStore,
};
use rand::rngs::OsRng;
use rstest::rstest;
use sam_client::storage::{key_generation::KyberKeyGenerator as _, SamStore, SamStoreType};
use sam_client::storage::{SignalStore, SignalStoreType};

#[rstest]
#[case(signal_in_mem().await.kyber_pre_key_store)]
#[case(signal_sqlite().await.kyber_pre_key_store)]
#[tokio::test]
async fn saved_kyber_pre_key_can_be_retrieved(
    #[case] mut kyber_pre_key_store: impl KyberPreKeyStore,
) {
    let id = 1.into();
    let mut csprng = OsRng;
    let identity_key = IdentityKeyPair::generate(&mut csprng);
    let kyber_pre_key_record =
        KyberPreKeyRecord::generate(KeyType::Kyber1024, id, identity_key.private_key())
            .expect("should be able to generate a Kyber pre key");

    kyber_pre_key_store
        .save_kyber_pre_key(kyber_pre_key_record.id().unwrap(), &kyber_pre_key_record)
        .await
        .unwrap();

    let retrived_record = kyber_pre_key_store
        .get_kyber_pre_key(kyber_pre_key_record.id().unwrap())
        .await
        .unwrap();

    assert_eq!(
        retrived_record.id().unwrap(),
        kyber_pre_key_record.id().unwrap()
    );

    assert_eq!(
        retrived_record.public_key().unwrap().serialize(),
        kyber_pre_key_record
            .key_pair()
            .unwrap()
            .public_key
            .serialize()
    );

    assert_eq!(
        retrived_record.secret_key().unwrap().serialize(),
        kyber_pre_key_record
            .key_pair()
            .unwrap()
            .secret_key
            .serialize()
    );
}

#[rstest]
#[case(signal_in_mem().await)]
#[case(signal_sqlite().await)]
#[tokio::test]
async fn kyber_pre_keys_ids_are_generated_properly(#[case] mut store: SignalStore<impl SignalStoreType>) {
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
                .kyber_pre_key_store
                .generate_key(id_key_pair.private_key())
                .await
                .expect("Can generate keys")
                .id()
                .expect("Can get id of key")
                .into(),
        );
    }

    assert_eq!(expected, ids)
}
