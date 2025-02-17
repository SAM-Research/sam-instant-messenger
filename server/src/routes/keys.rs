use axum::{
    extract::{Path, State},
    Json,
};
use sam_common::api::keys::{PreKeyBundles, PublishPreKeys};
use uuid::Uuid;

use crate::{
    auth::authenticated_user::AuthenticatedUser,
    logic::keys::{get_keybundles, publish_keybundle},
    state::{state_type::StateType, ServerState},
    ServerError,
};

/// Returns key bundles for users devices
pub async fn keys_bundles_endpoint<T: StateType>(
    Path(account_id): Path<Uuid>,
    State(state): State<ServerState<T>>,
) -> Result<Json<PreKeyBundles>, ServerError> {
    get_keybundles(&state, &account_id).await.map(Json)
}

/// Handle publish of new key bundles
pub async fn publish_keys_endpoint<T: StateType>(
    State(state): State<ServerState<T>>,
    auth_user: AuthenticatedUser,
    Json(req): Json<PublishPreKeys>,
) -> Result<(), ServerError> {
    publish_keybundle(
        &state,
        auth_user.account().id(),
        &auth_user.device().id(),
        req,
    )
    .await
}
