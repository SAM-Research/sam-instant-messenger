use axum::{
    extract::{Path, State},
    Json,
};
use axum_extra::{
    headers::{authorization::Basic, Authorization},
    TypedHeader,
};

use sam_common::{
    address::DeviceId,
    api::device::{LinkDeviceRequest, LinkDeviceResponse, LinkDeviceToken},
};

use crate::{
    auth::authenticated_user::AuthenticatedUser,
    logic::device::{create_device_token, link_device, unlink_device},
    state::{state_type::StateType, ServerState},
    ServerError,
};

/// Handle device provisioning
pub async fn device_provision_token_endpoint<T: StateType>(
    State(state): State<ServerState<T>>,
    auth_user: AuthenticatedUser,
) -> Result<Json<LinkDeviceToken>, ServerError> {
    if auth_user.device().id() != 1.into() {
        return Err(ServerError::DeviceProvisionUnAuth);
    }
    create_device_token(&state, auth_user.account().id())
        .await
        .map(Json)
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
    Path(device_id): Path<DeviceId>,
    auth_user: AuthenticatedUser,
) -> Result<(), ServerError> {
    unlink_device(&state, auth_user.account().id(), device_id).await
}
