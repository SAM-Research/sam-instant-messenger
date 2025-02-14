use crate::storage::tests::{in_mem, sqlite};

use super::alice_address;
use libsignal_protocol::SessionRecord;
use libsignal_protocol::SessionStore as _;
macro_rules! test_session_store {
    ( [ $( ($struct:ty, $factory:expr) ),* ]) => {
        $(
            paste::paste! {
                #[tokio::test]
                async fn [< $struct _load_and_store_session >]() {
                    let mut session_store = $factory().await.session_store;
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

                #[tokio::test]
                async fn [< $struct _no_session_in_new_store >]() {
                    let session_store = $factory().await.session_store;
                    let address = alice_address();

                    assert!(session_store
                        .load_session(&address)
                        .await
                        .unwrap()
                        .is_none()
                    );
                }
            }
        )*
    };
}
test_session_store!([
    (sqlite_session_store, sqlite),
    (in_memory_session_store, in_mem)
]);
