use std::collections::HashSet;

use super::{in_mem, sqlite};
use rstest::rstest;
use sam_client::storage::ContactStore;
use sam_common::address::AccountId;

#[rstest]
#[case(in_mem().await.contact_store)]
#[case(sqlite().await.contact_store)]
#[tokio::test]
async fn contact_device_can_be_stored_and_retrieved(#[case] mut contact_store: impl ContactStore) {
    let account_id = AccountId::generate();
    let device_id = 1.into();
    assert!(contact_store
        .get_all_devices(account_id)
        .await
        .is_ok_and(|devices| devices.is_empty()));
    assert!(contact_store
        .add_device(account_id, device_id)
        .await
        .is_ok());
    assert!(contact_store
        .get_all_devices(account_id)
        .await
        .is_ok_and(|devices| devices == HashSet::from([device_id])));
}

#[rstest]
#[case(in_mem().await.contact_store)]
#[case(sqlite().await.contact_store)]
#[tokio::test]
async fn contact_device_can_be_deleted(#[case] mut contact_store: impl ContactStore) {
    let account_id = AccountId::generate();
    let device_id = 1.into();

    assert!(contact_store
        .add_device(account_id, device_id)
        .await
        .is_ok());

    assert!(contact_store
        .remove_device(account_id, device_id)
        .await
        .is_ok());

    assert!(contact_store
        .get_all_devices(account_id)
        .await
        .is_ok_and(|devices| devices.is_empty()));
}
