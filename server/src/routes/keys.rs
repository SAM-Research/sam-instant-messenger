use crate::logic::keys::{get_keybundles, get_keybundles_for_all_devices};
use crate::routes::error::RouterError;
use crate::{
    auth::authenticated_user::AuthenticatedUser,
    logic::keys::publish_keybundle,
    state::{state_type::StateType, ServerState},
    ServerError,
};
use axum::extract::rejection::JsonRejection;
use axum::http::header::CONTENT_TYPE;
use axum::http::HeaderMap;
use axum::{
    extract::{Path, State},
    routing::{get, put},
    Json, Router,
};
use sam_common::{
    address::AccountId,
    api::keys::{PreKeyBundles, PublishPreKeys},
    DeviceId,
};

/// Returns key bundles for users devices
async fn key_bundles_for_some_devices_endpoint<T: StateType>(
    State(mut state): State<ServerState<T>>,
    Path(account_id): Path<AccountId>,
    headers: HeaderMap,
    auth_user: AuthenticatedUser,
    json: Result<Json<Vec<DeviceId>>, JsonRejection>,
) -> Result<Json<PreKeyBundles>, ServerError> {
    let content_type = headers.get(CONTENT_TYPE);
    match json {
        Err(JsonRejection::MissingJsonContentType(err)) => {
            if let Some(_) = content_type {
                return Err(
                    RouterError::JsonRejection(JsonRejection::MissingJsonContentType(err)).into(),
                );
            }
            get_keybundles_for_all_devices(
                &mut state,
                account_id,
                auth_user.account().id(),
                auth_user.device().id(),
            )
            .await
        }
        Err(err) => Err(RouterError::JsonRejection(err).into()),
        Ok(Json(device_ids)) => {
            if device_ids.is_empty() {
                return Err(RouterError::NoDeviceIdsInRequest)?;
            };
            get_keybundles(
                &mut state,
                account_id,
                device_ids,
                auth_user.account().id(),
                auth_user.device().id(),
            )
            .await
        }
    }
    .map(Json)
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
        .route(
            "/api/v1/keys/{account_id}",
            get(key_bundles_for_some_devices_endpoint),
        )
        .route("/api/v1/keys", put(publish_keys_endpoint))
}

#[cfg(test)]
mod test {
    use crate::{
        logic::keys::add_keybundle,
        routes::{
            keys::key_routes,
            test_utils::{create_user, test_server},
        },
        state::ServerState,
        test_utils::create_publish_pre_keys,
    };
    use axum::http;
    use base64::{prelude::BASE64_STANDARD, Engine};
    use rand::rngs::OsRng;
    use rstest::rstest;
    use sam_common::api::{keys::PreKeyBundles, PreKeyBundle};
    use sam_common::DeviceId;

    #[tokio::test]
    async fn test_post_api_v1_keys() {
        let mut state = ServerState::in_memory_test();
        let (pair, account_id, _) = create_user(&mut state, "alice", "phone", "bob", OsRng).await;

        let server = test_server(state, key_routes);
        let basic = format!(
            "Basic {}",
            BASE64_STANDARD.encode(format!("{}.1:{}", account_id, "bob"))
        );

        let res = server
            .put("/api/v1/keys")
            .add_header(http::header::AUTHORIZATION, basic)
            .json(&create_publish_pre_keys(
                Some(vec![1]),
                Some(3),
                Some(vec![4]),
                Some(33),
                &pair,
                OsRng,
            ))
            .await;
        res.assert_status_ok();
    }

    #[tokio::test]
    async fn test_get_api_v1_keys_account() {
        let mut state = ServerState::in_memory_test();
        let (pair, account_id, device_id) =
            create_user(&mut state, "alice", "phone", "bob", OsRng).await;
        let (_, bob_id, _) = create_user(&mut state, "bob", "phone", "password", OsRng).await;

        let keys = create_publish_pre_keys(
            Some(vec![1]),
            Some(3),
            Some(vec![4]),
            Some(33),
            &pair,
            OsRng,
        );
        add_keybundle(
            &mut state,
            pair.identity_key(),
            account_id,
            device_id,
            keys.clone(),
        )
        .await
        .expect("Can add keys");

        let server = test_server(state, key_routes);
        let basic = format!(
            "Basic {}",
            BASE64_STANDARD.encode(format!("{}.1:{}", bob_id, "password"))
        );

        let res = server
            .get(&format!("/api/v1/keys/{account_id}"))
            .add_header(http::header::AUTHORIZATION, basic)
            .await;

        let expected = PreKeyBundles {
            identity_key: *pair.identity_key(),
            bundles: vec![PreKeyBundle {
                device_id: 1,
                registration_id: 1,
                pre_key: keys.pre_keys.unwrap().first().cloned(),
                pq_pre_key: keys.pq_pre_keys.unwrap().first().cloned().unwrap(),
                signed_pre_key: keys.signed_pre_key.unwrap(),
            }],
        };

        res.assert_status_ok();
        res.assert_json(&expected);
    }

    #[rstest]
    #[case("garbage/content", 415)]
    #[case("application/json", 400)]
    #[tokio::test]
    async fn test_get_api_v1_keys_account_device_wrong_content_type(
        #[case] content_type: &str,
        #[case] expected_err: u16,
    ) {
        let mut state = ServerState::in_memory_test();
        let (pair, account_id, device_id) =
            create_user(&mut state, "alice", "phone", "bob", OsRng).await;
        let (_, bob_id, _) = create_user(&mut state, "bob", "phone", "password", OsRng).await;

        let keys = create_publish_pre_keys(
            Some(vec![1]),
            Some(3),
            Some(vec![4]),
            Some(33),
            &pair,
            OsRng,
        );
        add_keybundle(
            &mut state,
            pair.identity_key(),
            account_id,
            device_id,
            keys.clone(),
        )
        .await
        .expect("Can add keys");

        let server = test_server(state, key_routes);
        let basic = format!(
            "Basic {}",
            BASE64_STANDARD.encode(format!("{}.1:{}", bob_id, "password"))
        );

        let res = server
            .get(&format!("/api/v1/keys/{account_id}"))
            .add_header(http::header::AUTHORIZATION, basic)
            .add_header(http::header::CONTENT_TYPE, content_type)
            .await;

        assert_eq!(res.status_code().as_u16(), expected_err);
    }

    #[tokio::test]
    async fn test_get_api_v1_keys_account_device() {
        let mut state = ServerState::in_memory_test();
        let (pair, account_id, device_id) =
            create_user(&mut state, "alice", "phone", "bob", OsRng).await;
        let (_, bob_id, _) = create_user(&mut state, "bob", "phone", "password", OsRng).await;
        let keys = create_publish_pre_keys(
            Some(vec![1]),
            Some(3),
            Some(vec![4]),
            Some(33),
            &pair,
            OsRng,
        );
        add_keybundle(
            &mut state,
            pair.identity_key(),
            account_id,
            device_id,
            keys.clone(),
        )
        .await
        .expect("Can add keys");

        let server = test_server(state, key_routes);
        let basic = format!(
            "Basic {}",
            BASE64_STANDARD.encode(format!("{}.1:{}", bob_id, "password"))
        );

        let device_ids: Vec<DeviceId> = vec![1.into()];

        let res = server
            .get(&format!("/api/v1/keys/{account_id}"))
            .add_header(http::header::AUTHORIZATION, basic)
            .json(&device_ids)
            .await;

        let expected = PreKeyBundles {
            bundles: vec![PreKeyBundle {
                device_id: 1,
                registration_id: 1,
                pre_key: keys.pre_keys.unwrap().first().cloned(),
                pq_pre_key: keys.pq_pre_keys.unwrap().first().cloned().unwrap(),
                signed_pre_key: keys.signed_pre_key.unwrap(),
            }],
            identity_key: *pair.identity_key(),
        };

        res.assert_status_ok();
        res.assert_json(&expected);
    }
}
