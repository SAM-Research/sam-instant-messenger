use axum::{
    extract::{Path, State},
    Json,
};
use axum_extra::{
    headers::{authorization::Basic, Authorization},
    TypedHeader,
};
use sam_common::api::device::{LinkDeviceRequest, LinkDeviceResponse, LinkDeviceToken};

use crate::{
    auth::{authenticated_user::AuthenticatedUser, device::create_token},
    logic::device::{link_device, unlink_device},
    state::{
        traits::{device_manager::DeviceManager, state_type::StateType},
        ServerState,
    },
    ServerError,
};

/// Handle device provisioning
pub async fn device_provision_token_endpoint<T: StateType>(
    State(state): State<ServerState<T>>,
    auth_user: AuthenticatedUser,
) -> Result<Json<LinkDeviceToken>, ServerError> {
    if auth_user.device().id() != 1 {
        return Err(ServerError::DeviceProvisionUnAuth);
    }
    let devices = state.devices.lock().await;
    Ok(Json(create_token(
        devices.link_secret().await?,
        auth_user.account().id(),
    )))
}

/// Handle device linking
pub async fn link_device_endpoint<T: StateType>(
    State(state): State<ServerState<T>>,
    TypedHeader(Authorization(basic)): TypedHeader<Authorization<Basic>>,
    Json(req): Json<LinkDeviceRequest>,
) -> Result<Json<LinkDeviceResponse>, ServerError> {
    link_device(&state, req, basic.password().to_string())
        .await
        .map(Json)
}

/// Handle device linking
pub async fn delete_device_endpoint<T: StateType>(
    State(state): State<ServerState<T>>,
    Path(device_id): Path<u32>,
    auth_user: AuthenticatedUser,
) -> Result<(), ServerError> {
    unlink_device(&state, auth_user.account().id(), device_id).await
}
