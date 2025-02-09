use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    Json,
};
use libsignal_protocol::{Aci, ProtocolAddress};
use sam_common::{
    api::{
        account::{RegistrationRequest, RegistrationResponse},
        authorization::BasicAuthorizationHeader,
    },
    time_now_u128,
};
use uuid::Uuid;

use crate::{
    auth::{
        authenticate::{auth_header, SaltedTokenHash},
        authenticated_device::AuthenticatedDevice,
    },
    error::HTTPError,
    routes::keys::add_key_bundle,
    state::{account::Account, device::Device, ServerState},
};

/// Handle registration of new users
#[axum::debug_handler]
pub async fn account_register_endpoint(
    State(mut state): State<ServerState>,
    headers: HeaderMap,
    Json(registration): Json<RegistrationRequest>,
) -> Result<Json<RegistrationResponse>, HTTPError> {
    let auth_header: BasicAuthorizationHeader = auth_header(&headers)?;

    let username = auth_header.username();

    let aci: Aci = Uuid::new_v4().into();
    let hash = SaltedTokenHash::generate_for(auth_header.password())?;
    let device = Device::builder()
        .registration_id(registration.device_activation.registration_id)
        .device_id(1.into())
        .name(registration.device_activation.device_name)
        .created(time_now_u128())
        .auth_token(hash.hash())
        .salt(hash.salt())
        .build();
    let protocol_addr = ProtocolAddress::new(aci.service_id_string(), device.device_id);

    let account = Account::builder()
        .aci(aci)
        .username(username.to_string())
        .identity_key(registration.identity_key)
        .primary_device(device.device_id)
        .build();
    state.accounts.lock().await.add_account(account).await;
    state.accounts.lock().await.add_device(&aci, device).await;

    add_key_bundle(
        &mut state,
        &registration.identity_key,
        &protocol_addr,
        registration.device_activation.key_bundle,
    )
    .await;

    Ok(RegistrationResponse {
        account_id: aci.into(),
    })
    .map(Json)
}

// Handle deletion of account
pub async fn delete_account_endpoint(
    State(state): State<ServerState>,
    authenticated_device: AuthenticatedDevice,
) -> Result<(), HTTPError> {
    state
        .accounts
        .lock()
        .await
        .remove_account(authenticated_device.account.aci)
        .await
        .map_err(|e| HTTPError {
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
            body: e.to_string(),
        })
}
