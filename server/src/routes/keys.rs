use axum::{
    extract::{Path, State},
    Json,
};

use crate::{error::HTTPError, state::ServerState};
use sam_common::api::keys::{BundleResponse, PublishKeyBundleRequest};

/// Returns key bundles for users devices
pub async fn keys_bundles_endpoint(
    Path(account_id): Path<String>,
    State(state): State<ServerState>,
) -> Result<Json<BundleResponse>, HTTPError> {
    todo!()
}

/// Handle publish of new key bundles
pub async fn publish_keys_endpoint(
    State(state): State<ServerState>,
    Json(bundles): Json<PublishKeyBundleRequest>,
) -> Result<(), HTTPError> {
    todo!()
}
