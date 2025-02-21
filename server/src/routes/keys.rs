use axum::{
    extract::{Path, State},
    routing::{get, put},
    Json, Router,
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

/// Returns key bundles for users devices
pub async fn keys_bundles_endpoint<T: StateType>(
    Path(account_id): Path<AccountId>,
    State(mut state): State<ServerState<T>>,
) -> Result<Json<PreKeyBundles>, ServerError> {
    get_keybundles(&mut state, account_id).await.map(Json)
}

/// Handle publish of new key bundles
async fn publish_keys_endpoint<T: StateType>(
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

pub fn key_routes<T: StateType>(router: Router<ServerState<T>>) -> Router<ServerState<T>> {
    router
        .route("/api/v1/keys/{account_id}", get(keys_bundles_endpoint))
        .route("/api/v1/keys", put(publish_keys_endpoint))
}

#[cfg(test)]
mod test {
    use axum::http::{self, StatusCode};
    use base64::{prelude::BASE64_STANDARD, Engine};
    use rand::rngs::OsRng;

    use crate::{
        managers::in_memory::test_utils::LINK_SECRET,
        routes::{
            keys::key_routes,
            test_utils::{create_user, test_server},
        },
        state::ServerState,
        test_utils::create_publish_key_bundle,
    };

    #[tokio::test]
    async fn test_publish_keys() {
        let mut state = ServerState::in_memory(LINK_SECRET.to_owned(), 10);
        let (pair, account_id, _) = create_user(&mut state, "alice", "phone", "bob", OsRng).await;

        let server = test_server(state, key_routes);
        let basic = format!(
            "Basic {}",
            BASE64_STANDARD.encode(format!("{}.1:{}", account_id, "bob"))
        );

        let res = server
            .put("/api/v1/keys")
            .add_header(http::header::AUTHORIZATION, basic)
            .json(&create_publish_key_bundle(
                Some(vec![1]),
                Some(3),
                Some(vec![4]),
                Some(33),
                &pair,
                OsRng,
            ))
            .await;
        res.assert_status(StatusCode::OK);
    }
}
