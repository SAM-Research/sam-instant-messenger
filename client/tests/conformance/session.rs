use super::{signal_in_mem, signal_sqlite};

use super::alice_address;
use libsignal_protocol::{SessionRecord, SessionStore};
use rstest::rstest;

#[rstest]
#[case(signal_in_mem().await.session_store)]
#[case(signal_sqlite().await.session_store)]
#[tokio::test]
async fn load_and_store_session(#[case] mut session_store: impl SessionStore) {
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

#[rstest]
#[case(signal_in_mem().await.session_store)]
#[case(signal_sqlite().await.session_store)]
#[tokio::test]
async fn no_session_in_new_store(#[case] session_store: impl SessionStore) {
    let address = alice_address();

    assert!(session_store
        .load_session(&address)
        .await
        .unwrap()
        .is_none());
}
