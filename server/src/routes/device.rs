use axum::{
    extract::{Path, State},
    routing::{delete, get, post},
    Json, Router,
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
async fn device_provision_token_endpoint<T: StateType>(
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
async fn link_device_endpoint<T: StateType>(
    State(mut state): State<ServerState<T>>,
    TypedHeader(Authorization(basic)): TypedHeader<Authorization<Basic>>,
    Json(req): Json<LinkDeviceRequest>,
) -> Result<Json<LinkDeviceResponse>, ServerError> {
    link_device(&mut state, req, basic.password().to_string())
        .await
        .map(Json)
}

/// Handle device linking
async fn delete_device_endpoint<T: StateType>(
    State(mut state): State<ServerState<T>>,
    Path(device_id): Path<DeviceId>,
    auth_user: AuthenticatedUser,
) -> Result<(), ServerError> {
    unlink_device(&mut state, auth_user.account().id(), device_id).await
}

pub fn device_routes<T: StateType>(router: Router<ServerState<T>>) -> Router<ServerState<T>> {
    router
        .route(
            "/api/v1/devices/provision",
            get(device_provision_token_endpoint),
        )
        .route("/api/v1/devices/link", post(link_device_endpoint))
        .route("/api/v1/device/{id}", delete(delete_device_endpoint))
}
