use libsignal_protocol::IdentityKey;
use sam_common::{
    api::device::{DeviceActivationInfo, LinkDeviceRequest, LinkDeviceResponse},
    time_now_u128,
};
use uuid::Uuid;

use crate::{
    auth::{device::verify_token, password::Password},
    state::{
        entities::device::Device,
        traits::{
            account_manager::AccountManager, device_manager::DeviceManager, state_type::StateType,
        },
        ServerState,
    },
    ServerError,
};

use super::keys::add_keybundle;

pub async fn link_device<T: StateType>(
    state: &ServerState<T>,
    device_link: LinkDeviceRequest,
    password: String,
) -> Result<LinkDeviceResponse, ServerError> {
    let account_id = {
        let devices = state.devices.lock().await;
        verify_token(devices.link_secret().await?, device_link.token)?
    };

    let account = {
        let accounts = state.accounts.lock().await;
        accounts.get_account(&account_id).await?
    };

    let devices = state.devices.lock().await;
    let next_id = devices.next_device_id(&account_id).await?;
    create_device(
        &state,
        &account_id,
        account.identity(),
        device_link.device_activation,
        next_id,
        password,
    )
    .await?;

    Ok(LinkDeviceResponse {
        account_id,
        device_id: next_id,
    })
}

pub async fn unlink_device<T: StateType>(
    state: &ServerState<T>,
    account_id: &Uuid,
    device_id: u32,
) -> Result<(), ServerError> {
    state
        .devices
        .lock()
        .await
        .remove_device(account_id, device_id)
        .await
}

pub async fn create_device<T: StateType>(
    state: &ServerState<T>,
    account_id: &Uuid,
    identity: &IdentityKey,
    device_info: DeviceActivationInfo,
    device_id: u32,
    password: String,
) -> Result<(), ServerError> {
    let device = Device::builder()
        .id(device_id)
        .registration_id(device_info.registration_id)
        .name(device_info.name)
        .creation(time_now_u128())
        .password(Password::generate(password)?)
        .build();

    state
        .devices
        .lock()
        .await
        .add_device(&account_id, device)
        .await?;

    add_keybundle(
        state,
        identity,
        account_id,
        &device_id,
        device_info.key_bundle,
    )
    .await
}
