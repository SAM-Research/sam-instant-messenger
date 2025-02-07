use axum::{
    extract::{Path, State},
    Json,
};
use sam_common::api::device::{LinkDeviceRequest, LinkDeviceResponse, LinkDeviceToken};

use crate::{error::HTTPError, state::ServerState};

/// Handle device provisioning
pub async fn provision_device_endpoint(
    State(state): State<ServerState>,
) -> Result<Json<LinkDeviceToken>, HTTPError> {
    todo!()
}

/// Handle device linking
pub async fn link_device_endpoint(
    State(state): State<ServerState>,
    Json(link_device_request): Json<LinkDeviceRequest>,
) -> Result<Json<LinkDeviceResponse>, HTTPError> {
    todo!()
}

/// Handle device linking
pub async fn delete_device_endpoint(Path(device_id): Path<String>) {
    todo!()
}
