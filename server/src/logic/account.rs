use sam_common::api::account::{RegistrationRequest, RegistrationResponse};
use uuid::Uuid;

use crate::{
    logic::device::create_device,
    managers::{
        entities::account::Account,
        traits::{
            account_manager::AccountManager, device_manager::DeviceManager,
            key_manager::KeyManager, message_manager::MessageManager,
        },
    },
    state::{state_type::StateType, ServerState},
    ServerError,
};

pub async fn delete_account<T: StateType>(
    state: ServerState<T>,
    account_id: Uuid,
) -> Result<(), ServerError> {
    {
        let mut keys = state.keys.lock().await;
        keys.remove_account_keys(&account_id).await?;
    }

    {
        let mut messages = state.messages.lock().await;
        let mut devices = state.devices.lock().await;
        for device_id in devices.get_devices(&account_id).await? {
            for msg_id in messages.get_messages(&account_id, &device_id).await? {
                messages
                    .remove_message(&account_id, &device_id, msg_id)
                    .await?;
            }
            devices.remove_device(&account_id, device_id).await?;
        }
    }

    let mut accounts = state.accounts.lock().await;
    accounts.remove_account(account_id).await
}

pub async fn create_account<T: StateType>(
    state: &ServerState<T>,
    registration: RegistrationRequest,
    username: String,
    password: String,
) -> Result<RegistrationResponse, ServerError> {
    let account = Account::builder()
        .id(Uuid::new_v4())
        .username(username)
        .identity(registration.identity_key)
        .build();

    state.accounts.lock().await.add_account(&account).await?;

    create_device(
        state,
        account.id(),
        account.identity(),
        registration.device_activation,
        1,
        password,
    )
    .await?;
    Ok(RegistrationResponse {
        account_id: *account.id(),
    })
}
