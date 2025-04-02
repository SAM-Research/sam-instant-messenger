use super::{signal_in_mem, signal_sqlite};
use libsignal_protocol::{KeyPair, PreKeyRecord, PreKeyStore};
use rand::rngs::OsRng;
use rand::{rngs::StdRng, SeedableRng as _};
use rstest::rstest;
use sam_client::storage::key_generation::PreKeyGenerator;

#[rstest]
#[case(signal_in_mem().await.pre_key_store)]
#[case(signal_sqlite().await.pre_key_store)]
#[tokio::test]
async fn pre_key_can_be_saved_and_retrieved(#[case] mut pre_key_store: impl PreKeyStore) {
    let id = 0.into();
    let mut csprng = OsRng;
    let pre_key_record = PreKeyRecord::new(id, &KeyPair::generate(&mut csprng));

    pre_key_store
        .save_pre_key(pre_key_record.id().unwrap(), &pre_key_record)
        .await
        .unwrap();

    let retrived_pre_key = pre_key_store.get_pre_key(id).await.unwrap();

    assert_eq!(retrived_pre_key.id().unwrap(), pre_key_record.id().unwrap());

    assert_eq!(
        retrived_pre_key.public_key().unwrap(),
        pre_key_record.key_pair().unwrap().public_key
    );

    assert_eq!(
        retrived_pre_key.private_key().unwrap().serialize(),
        pre_key_record.key_pair().unwrap().private_key.serialize()
    );
}

#[rstest]
#[case(signal_in_mem().await.pre_key_store)]
#[case(signal_sqlite().await.pre_key_store)]
#[tokio::test]
async fn pre_key_can_be_removed(#[case] mut pre_key_store: impl PreKeyStore) {
    let id = 0.into();
    let mut csprng = OsRng;
    let pre_key_record = PreKeyRecord::new(id, &KeyPair::generate(&mut csprng));

    pre_key_store
        .save_pre_key(pre_key_record.id().unwrap(), &pre_key_record)
        .await
        .unwrap();

    let _ = pre_key_store.get_pre_key(id).await.unwrap();

    pre_key_store
        .remove_pre_key(pre_key_record.id().unwrap())
        .await
        .unwrap();

    pre_key_store
        .get_pre_key(pre_key_record.id().unwrap())
        .await
        .expect_err("We should not be able to retrive the key after deletion");
}

#[rstest]
#[case(signal_in_mem().await.pre_key_store)]
#[case(signal_sqlite().await.pre_key_store)]
#[tokio::test]
async fn pre_keys_ids_are_generated_properly(
    #[case] mut generator: impl PreKeyStore + PreKeyGenerator,
) {
    let mut rng = StdRng::seed_from_u64(128);
    let expected: Vec<u32> = (1u32..=10u32).collect();

    let mut ids: Vec<u32> = Vec::new();

    for _ in 1u32..=10u32 {
        ids.push(
            generator
                .generate_key(&mut rng)
                .await
                .expect("Can generate keys")
                .id()
                .expect("Can get id of key")
                .into(),
        );
    }

    assert_eq!(expected, ids)
}
