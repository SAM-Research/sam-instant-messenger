use axum::{
    extract::{Path, State},
    Json,
};
use sam_common::{
    address::AccountId,
    api::keys::{PreKeyBundles, PublishPreKeys},
};

use crate::{
    auth::authenticated_user::AuthenticatedUser,
    logic::keys::{get_keybundles, publish_keybundle},
    state::{state_type::StateType, ServerState},
    ServerError,
};

/*
#[utoipa::path(
    get,
    path = "/api/v1/keys/{account_id}",
    responses(
        (status = 200, description = "Got key bundle successfully", body = PreKeyBundles, content_type = "application/json"),
        (status = 500, description = "Was unsuccessful to get key bundle")
    ),
    params(
        ("account_id" = AccountId, Path, description = "The id of the account you want to get keys for"),
    ),
)]

 */
pub async fn keys_bundles_endpoint<T: StateType>(
    Path(account_id): Path<AccountId>,
    State(mut state): State<ServerState<T>>,
) -> Result<Json<PreKeyBundles>, ServerError> {
    get_keybundles(&mut state, account_id).await.map(Json)
}

/*
#[utoipa::path(
    put,
    path = "/api/v1/keys",
    request_body(content = PublishPreKeys, content_type = "application/json"),
    responses(
        (status = 200, description = "Successfully published keys"),
        (status = 500, description = "Unsuccessfully published keys"),
    ),
    params(
        ("auth_user" = AuthenticatedUser, Path, description = "Authentication"),
    ),
)]

 */
pub async fn publish_keys_endpoint<T: StateType>(
    State(mut state): State<ServerState<T>>,
    auth_user: AuthenticatedUser,
    Json(req): Json<PublishPreKeys>,
) -> Result<(), ServerError> {
    publish_keybundle(
        &mut state,
        auth_user.account().id(),
        auth_user.device().id(),
        req,
    )
    .await
}
