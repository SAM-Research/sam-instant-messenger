use crate::storage::tests::{in_mem, sqlite};

use super::alice_address;
use libsignal_protocol::{SessionRecord, SessionStore};

macro_rules! test_session_store {
    ( [ $( ($struct:ty, $factory:expr) ),* ]) => {
        $(
            paste::paste! {
                #[tokio::test]
                async fn [< $struct _load_and_store_session >]() {
                    load_and_store_session($factory().await.session_store).await;
                }

                #[tokio::test]
                async fn [< $struct _no_session_in_new_store >]() {
                    no_session_in_new_store($factory().await.session_store).await;
                }
            }
        )*
    };
}

async fn load_and_store_session(mut session_store: impl SessionStore) {
    let address = alice_address();
    let record = SessionRecord::new_fresh();
    session_store
        .store_session(&address, &record)
        .await
        .unwrap();

    assert_eq!(
        session_store
            .load_session(&address)
            .await
            .unwrap()
            .unwrap()
            .serialize()
            .unwrap(),
        record.serialize().unwrap()
    );
}

async fn no_session_in_new_store(session_store: impl SessionStore) {
    let address = alice_address();

    assert!(session_store
        .load_session(&address)
        .await
        .unwrap()
        .is_none());
}

test_session_store!([
    (sqlite_session_store, sqlite),
    (in_memory_session_store, in_mem)
]);
