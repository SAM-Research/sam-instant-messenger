use libsignal_protocol::{
    kem::KeyType, GenericSignedPreKey as _, IdentityKeyPair, KyberPreKeyRecord, KyberPreKeyStore,
};
use rand::rngs::OsRng;

use super::{in_mem, sqlite};

macro_rules! test_kyber_key_store {
    ( [ $( ($struct:ty, $factory:expr) ),* ]) => {
        $(
            paste::paste! {
                #[tokio::test]
                async fn [< $struct _saved_kyber_pre_key_can_be_retrieved >]() {
                    saved_kyber_pre_key_can_be_retrieved($factory().await.kyber_pre_key_store).await;
                }
            }
        )*
    };
}

async fn saved_kyber_pre_key_can_be_retrieved(mut kyber_pre_key_store: impl KyberPreKeyStore) {
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

test_kyber_key_store!([
    (sqlite_kyber_key_store, sqlite),
    (in_memory_kyber_key_store, in_mem)
]);
