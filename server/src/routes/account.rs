use axum::{
    extract::State,
    routing::{delete, post},
    Json, Router,
};

use axum_extra::{
    headers::{authorization::Basic, Authorization},
    TypedHeader,
};
use sam_common::api::account::{RegistrationRequest, RegistrationResponse};

use crate::{
    auth::authenticated_user::AuthenticatedUser,
    logic::account::{create_account, delete_account},
    state::{state_type::StateType, ServerState},
    ServerError,
};

/// Handle registration of new users
async fn account_register_endpoint<T: StateType>(
    State(mut state): State<ServerState<T>>,
    TypedHeader(Authorization(basic)): TypedHeader<Authorization<Basic>>,
    Json(req): Json<RegistrationRequest>,
) -> Result<Json<RegistrationResponse>, ServerError> {
    create_account(
        &mut state,
        req,
        basic.username().to_string(),
        basic.password().to_string(),
    )
    .await
    .map(Json)
}

// Handle deletion of account
async fn delete_account_endpoint<T: StateType>(
    State(mut state): State<ServerState<T>>,
    auth_user: AuthenticatedUser,
) -> Result<(), ServerError> {
    if *auth_user.device().id() != 1 {
        return Err(ServerError::DeviceUnAuth);
    }
    delete_account(&mut state, auth_user.account().id()).await
}

pub fn account_routes<T: StateType>(router: Router<ServerState<T>>) -> Router<ServerState<T>> {
    router
        .route("/api/v1/account", post(account_register_endpoint))
        .route("/api/v1/account", delete(delete_account_endpoint))
}

#[cfg(test)]
mod test {
    use axum::http;
    use base64::{prelude::BASE64_STANDARD, Engine};
    use libsignal_protocol::IdentityKeyPair;
    use rand::rngs::OsRng;
    use sam_common::api::{
        device::DeviceActivationInfo, RegistrationRequest, RegistrationResponse,
    };

    use crate::{
        managers::traits::{
            account_manager::AccountManager,
            key_manager::{LastResortKeyManager, SignedPreKeyManager},
        },
        routes::{
            account::account_routes,
            test_utils::{create_user, test_server},
        },
        state::ServerState,
        test_utils::{create_publish_pre_keys, pq_pre_key, signed_ec_pre_key},
    };

    #[tokio::test]
    async fn test_post_api_v1_account() {
        let state = ServerState::in_memory_test();

        let server = test_server(state.clone(), account_routes);

        let id_pair = IdentityKeyPair::generate(&mut OsRng);
        let basic = format!("Basic {}", BASE64_STANDARD.encode("alice:password"));

        let reg_request = RegistrationRequest {
            identity_key: *id_pair.identity_key(),
            device_activation: DeviceActivationInfo {
                name: "phone".to_string(),
                registration_id: 1.into(),
                key_bundle: create_publish_pre_keys(
                    Some(vec![1]),
                    Some(3),
                    Some(vec![4]),
                    Some(33),
                    &id_pair,
                    OsRng,
                )
                .try_into()
                .expect("Can make RegistrationPreKeys"),
            },
        };

        let res = server
            .post("/api/v1/account")
            .add_header(http::header::AUTHORIZATION, basic)
            .json(&reg_request)
            .await;
        let json_res = res.json::<RegistrationResponse>();
        res.assert_status_ok();
        assert!(state
            .accounts
            .get_account(json_res.account_id)
            .await
            .is_ok());
    }

    #[tokio::test]
    async fn test_delete_api_v1_account() {
        let mut state = ServerState::in_memory_test();
        let (pair, account_id, device_id) =
            create_user(&mut state, "alice", "phone", "password", OsRng).await;

        state
            .keys
            .set_last_resort_key(
                account_id,
                device_id,
                pair.identity_key(),
                pq_pre_key(1, &pair),
            )
            .await
            .expect("Can set key");

        state
            .keys
            .set_signed_pre_key(
                account_id,
                device_id,
                pair.identity_key(),
                signed_ec_pre_key(2, &pair, OsRng),
            )
            .await
            .expect("Can set signed prekey");

        let server = test_server(state.clone(), account_routes);
        let basic = format!(
            "Basic {}",
            BASE64_STANDARD.encode(format!("{account_id}.1:password"))
        );

        let res = server
            .delete("/api/v1/account")
            .add_header(http::header::AUTHORIZATION, basic)
            .await;
        res.assert_status_ok();
    }
}
