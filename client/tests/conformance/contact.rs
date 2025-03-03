use std::collections::HashSet;

use super::{in_mem, sqlite};
use sam_client::storage::ContactStore;
use sam_common::address::AccountId;

macro_rules! test_contact_store {
    ( [ $( ($struct:ty, $factory:expr) ),* ]) => {
        $(
            paste::paste! {
                #[tokio::test]
                async fn [< $struct _contact_device_can_be_stored_and_retrieved >]() {
                    contact_device_can_be_stored_and_retrieved($factory().await.contact_store).await;
                }

                #[tokio::test]
                async fn [< $struct _contact_device_can_be_deleted >]() {
                    contact_device_can_be_deleted($factory().await.contact_store).await;
                }
            }
        )*
    };
}

async fn contact_device_can_be_stored_and_retrieved(mut contact_store: impl ContactStore) {
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

async fn contact_device_can_be_deleted(mut contact_store: impl ContactStore) {
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

test_contact_store!([
    (sqlite_contact_store, sqlite),
    (in_memory_contact_store, in_mem)
]);
