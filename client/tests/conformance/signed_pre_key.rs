use super::{in_mem, sqlite};
use libsignal_protocol::{
    GenericSignedPreKey as _, IdentityKeyPair, KeyPair, SignedPreKeyRecord, SignedPreKeyStore,
};
use rand::rngs::OsRng;
use sam_client::signal_time_now;

macro_rules! test_signed_pre_key_store {
    ( [ $( ($struct:ty, $factory:expr) ),* ]) => {
        $(
            paste::paste! {
                #[tokio::test]
                async fn [< $struct _signed_pre_key_can_be_saved_and_retrieved >]() {
                    signed_pre_key_can_be_saved_and_retrieved($factory().await.signed_pre_key_store).await;
                }
            }
        )*
    };
}

async fn signed_pre_key_can_be_saved_and_retrieved(
    mut signed_pre_key_store: impl SignedPreKeyStore,
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

test_signed_pre_key_store!([
    (sqlite_signed_pre_key_store, sqlite),
    (in_memory_signed_pre_key_store, in_mem)
]);
