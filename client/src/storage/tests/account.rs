use crate::storage::tests::{in_mem, sqlite};
use crate::storage::AccountStore;
use crate::ClientError;
use sam_common::address::AccountId;

macro_rules! test_account_store {
    ( [ $( ($struct:ty, $factory:expr) ),* ]) => {
        $(
            paste::paste! {
                #[tokio::test]
                async fn [< $struct _username_can_be_stored_and_retrieved >]() {
                    username_can_be_stored_and_retrieved($factory().await.account_store).await;
                }

                #[tokio::test]
                async fn [< $struct _password_can_be_stored_and_retrieved >]() {
                    password_can_be_stored_and_retrieved($factory().await.account_store).await;
                }

                #[tokio::test]
                async fn [< $struct _account_id_can_be_stored_and_retrieved >]() {
                    account_id_can_be_stored_and_retrieved($factory().await.account_store).await;
                }
            }
        )*
    };
}

async fn account_id_can_be_stored_and_retrieved(mut account_store: impl AccountStore) {
    let account_id = AccountId::generate();
    assert!(matches!(
        account_store.get_account_id().await.unwrap_err(),
        ClientError::NoAccountId
    ));
    assert!(account_store
        .set_account_id(account_id.to_owned())
        .await
        .is_ok());
    assert_eq!(account_store.get_account_id().await.unwrap(), account_id);
}

async fn password_can_be_stored_and_retrieved(mut account_store: impl AccountStore) {
    let password = "MyPassword".to_owned();
    assert!(matches!(
        account_store.get_password().await.unwrap_err(),
        ClientError::NoPassword
    ));
    assert!(account_store
        .set_password(password.to_owned())
        .await
        .is_ok());
    assert_eq!(account_store.get_password().await.unwrap(), password);
}

async fn username_can_be_stored_and_retrieved(mut account_store: impl AccountStore) {
    let username = "MyUsername".to_owned();
    assert!(matches!(
        account_store.get_username().await.unwrap_err(),
        ClientError::NoUsername
    ));
    assert!(account_store
        .set_username(username.to_owned())
        .await
        .is_ok());
    assert_eq!(account_store.get_username().await.unwrap(), username);
}

test_account_store!([
    (sqlite_account_store, sqlite),
    (in_memory_account_store, in_mem)
]);
