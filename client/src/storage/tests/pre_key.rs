use crate::storage::tests::{in_mem, sqlite};
use crate::test_pre_key_store;
use libsignal_protocol::{KeyPair, PreKeyRecord, PreKeyStore as _};
use rand::rngs::OsRng;

#[macro_export]
macro_rules! test_pre_key_store {
    ( [ $( ($struct:ty, $factory:expr) ),* ]) => {
        $(
            paste::paste! {
                #[tokio::test]
                async fn [< $struct _pre_key_can_be_saved_and_retrieved >]() {
                    let mut pre_key_store = $factory().await.pre_key_store;
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

                #[tokio::test]
                async fn [< $struct _pre_key_can_be_removed >]() {
                    let mut pre_key_store = $factory().await.pre_key_store;
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
            }
        )*
    };
}

test_pre_key_store!([
    (sqlite_pre_key_store, sqlite),
    (in_memory_pre_key_store, in_mem)
]);
