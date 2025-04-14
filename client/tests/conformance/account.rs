use super::{in_mem, sqlite};
use rstest::rstest;
use sam_client::storage::error::StoreError;
use sam_client::storage::AccountStore;
use sam_common::address::AccountId;
use sam_common::DeviceId;

#[rstest]
#[case(in_mem().await.account_store)]
#[case(sqlite().await.account_store)]
#[tokio::test]
async fn account_id_can_be_stored_and_retrieved(#[case] mut account_store: impl AccountStore) {
    let account_id = AccountId::generate();
    assert!(matches!(
        account_store.get_account_id().await.unwrap_err(),
        StoreError::NoAccountId
    ));
    assert!(account_store
        .set_account_id(account_id.to_owned())
        .await
        .is_ok());
    assert_eq!(account_store.get_account_id().await.unwrap(), account_id);
}

#[rstest]
#[case(in_mem().await.account_store)]
#[case(sqlite().await.account_store)]
#[tokio::test]
async fn password_can_be_stored_and_retrieved(#[case] mut account_store: impl AccountStore) {
    let password = "MyPassword".to_owned();
    assert!(matches!(
        account_store.get_password().await.unwrap_err(),
        StoreError::NoPassword
    ));
    assert!(account_store
        .set_password(password.to_owned())
        .await
        .is_ok());
    assert_eq!(account_store.get_password().await.unwrap(), password);
}

#[rstest]
#[case(in_mem().await.account_store)]
#[case(sqlite().await.account_store)]
#[tokio::test]
async fn username_can_be_stored_and_retrieved(#[case] mut account_store: impl AccountStore) {
    let username = "MyUsername".to_owned();
    assert!(matches!(
        account_store.get_username().await.unwrap_err(),
        StoreError::NoUsername
    ));
    assert!(account_store
        .set_username(username.to_owned())
        .await
        .is_ok());
    assert_eq!(account_store.get_username().await.unwrap(), username);
}

#[rstest]
#[case(in_mem().await.account_store)]
#[case(sqlite().await.account_store)]
#[tokio::test]
async fn device_id_can_be_stored_and_retrieved(
    #[case] mut account_store: impl AccountStore,
    #[values(0u32, 1u32, u32::MAX, 37480u32)] device_id: u32,
) {
    let device_id: DeviceId = device_id.into();
    assert!(matches!(
        account_store.get_device_id().await.unwrap_err(),
        StoreError::NoDeviceId
    ));
    assert!(account_store
        .set_device_id(device_id.to_owned())
        .await
        .is_ok());
    assert_eq!(account_store.get_device_id().await.unwrap(), device_id);
}
