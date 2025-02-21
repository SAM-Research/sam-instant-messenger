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
    if *auth_user.device().id() == 1 && auth_user.device().id() != device_id {
        return Err(ServerError::DeviceUnAuth);
    }
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

#[cfg(test)]
mod test {
    use std::time::Duration;

    use axum::http::{self, StatusCode};
    use base64::{prelude::BASE64_STANDARD, Engine};
    use rand::rngs::OsRng;
    use rstest::rstest;
    use sam_common::api::{device::DeviceActivationInfo, LinkDeviceRequest, LinkDeviceToken};

    use crate::{
        auth::password::Password,
        managers::{
            entities::device::Device, in_memory::test_utils::LINK_SECRET,
            traits::device_manager::DeviceManager,
        },
        routes::{
            device::device_routes,
            test_utils::{create_user, test_server},
        },
        state::ServerState,
        test_utils::create_publish_pre_keys,
    };

    #[tokio::test]
    async fn test_get_api_v1_devices_provision() {
        let mut state = ServerState::in_memory_test();

        let (_, account_id, _) = create_user(&mut state, "alice", "phone", "password", OsRng).await;

        let server = test_server(state.clone(), device_routes);
        let basic = format!(
            "Basic {}",
            BASE64_STANDARD.encode(format!("{account_id}.1:password"))
        );

        let res = server
            .get("/api/v1/devices/provision")
            .add_header(http::header::AUTHORIZATION, basic)
            .await;
        res.assert_status_ok();
    }

    #[rstest]
    #[case(600, StatusCode::OK, true)]
    #[case(0, StatusCode::FORBIDDEN, false)]
    #[tokio::test]
    async fn test_get_api_v1_devices_link(
        #[case] expire_time: u64,
        #[case] expected_status: StatusCode,
        #[case] expects_ok_device: bool,
    ) {
        let mut state = ServerState::in_memory(LINK_SECRET.to_string(), expire_time, 10);

        let (pair, account_id, _) =
            create_user(&mut state, "alice", "phone", "password", OsRng).await;

        let server = test_server(state.clone(), device_routes);
        let basic = format!(
            "Basic {}",
            BASE64_STANDARD.encode(format!("{account_id}.1:password"))
        );

        let res = server
            .get("/api/v1/devices/provision")
            .add_header(http::header::AUTHORIZATION, basic)
            .await;

        let token = res.json::<LinkDeviceToken>();
        let req = LinkDeviceRequest {
            token,
            device_activation: DeviceActivationInfo {
                name: "car".to_string(),
                registration_id: 2.into(),
                key_bundle: create_publish_pre_keys(None, Some(66), None, Some(33), &pair, OsRng)
                    .try_into()
                    .expect("Can make RegistrationPreKeys"),
            },
        };

        // this request is be made by provisioned device
        let basic = format!(
            "Basic {}",
            BASE64_STANDARD.encode(format!("{account_id}:otherpass"))
        );

        if expire_time == 0 {
            tokio::time::sleep(Duration::from_secs(2)).await
        }

        let res = server
            .post("/api/v1/devices/link")
            .add_header(http::header::AUTHORIZATION, basic)
            .json(&req)
            .await;
        res.assert_status(expected_status);
        assert!(state.devices.get_device(account_id, 2.into()).await.is_ok() == expects_ok_device);
    }

    #[tokio::test]
    async fn test_delete_api_v1_device_id() {
        let mut state = ServerState::in_memory_test();

        let (_, account_id, _) = create_user(&mut state, "alice", "phone", "password", OsRng).await;
        state
            .devices
            .add_device(
                account_id,
                &Device::builder()
                    .creation(0)
                    .id(2.into())
                    .registration_id(1.into())
                    .name("microwave".to_string())
                    .password(
                        Password::generate("otherpass".to_string())
                            .expect("Password can be generated"),
                    )
                    .build(),
            )
            .await
            .expect("Can Add Device");

        let server = test_server(state.clone(), device_routes);
        let basic = format!(
            "Basic {}",
            BASE64_STANDARD.encode(format!("{account_id}.2:otherpass"))
        );

        let res = server
            .delete("/api/v1/device/2")
            .add_header(http::header::AUTHORIZATION, basic)
            .await;
        res.assert_status_ok();
    }
}
