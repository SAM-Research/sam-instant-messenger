use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::{
    auth::{
        authenticate::{auth_header, service_id_aci, SaltedTokenHash},
        authenticated_device::AuthenticatedDevice,
    },
    error::HTTPError,
    routes::keys::add_key_bundle,
    state::{device::Device, ServerState},
};
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use base64::{
    prelude::{BASE64_STANDARD_NO_PAD, BASE64_URL_SAFE},
    Engine as _,
};
use hkdf::hmac::{Hmac, Mac};
use libsignal_protocol::{Aci, ProtocolAddress, ServiceId};
use sam_common::{
    api::{
        authorization::BasicAuthorizationHeader,
        device::{LinkDeviceRequest, LinkDeviceResponse, LinkDeviceToken},
    },
    time_now_u128,
};
use sha2::{Digest, Sha256};
use uuid::Uuid;

pub fn create_signature(link_secret: &str, claims: &str) -> Vec<u8> {
    let mut mac = Hmac::<Sha256>::new_from_slice(link_secret.as_bytes())
        .expect("HMAC can take key of any size");
    mac.update(claims.as_bytes());
    mac.finalize().into_bytes().to_vec()
}

/// Handle device provisioning
pub async fn device_provision_token_endpoint(
    State(state): State<ServerState>,
    authenticated_device: AuthenticatedDevice,
) -> Result<Json<LinkDeviceToken>, HTTPError> {
    if authenticated_device.device.device_id != 1.into() {
        return Err(HTTPError {
            status_code: StatusCode::UNAUTHORIZED,
            body: "".to_string(),
        });
    }

    let claims = format!(
        "{}.{}",
        authenticated_device.account.aci.service_id_string(),
        time_now_u128()
    );

    let signature = create_signature(&state.link_secret, &claims);

    let link_device_token = format!("{}:{}", claims, BASE64_URL_SAFE.encode(signature));
    let mut hasher = Sha256::new();
    hasher.update(link_device_token.as_bytes());
    let digest = hasher.finalize();
    let token_id = BASE64_STANDARD_NO_PAD.encode(digest);

    Ok(LinkDeviceToken {
        verification_code: link_device_token,
        token_identifier: token_id,
    })
    .map(Json)
}

/// Handle device linking
pub async fn link_device_endpoint(
    State(mut state): State<ServerState>,
    headers: HeaderMap,
    Json(link_device_request): Json<LinkDeviceRequest>,
) -> Result<Json<LinkDeviceResponse>, HTTPError> {
    let auth_header: BasicAuthorizationHeader = auth_header(&headers)?;
    let (claims, b64_signature) = link_device_request
        .verification_code
        .split_once(":")
        .ok_or(HTTPError {
            status_code: StatusCode::FORBIDDEN,
            body: "".to_owned(),
        })?;
    let expected_signature = create_signature(&state.link_secret, &claims);
    let signature = BASE64_URL_SAFE
        .decode(b64_signature)
        .map_err(|_| HTTPError {
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
            body: "".to_owned(),
        })?;

    if expected_signature.as_slice() != signature {
        return Err(HTTPError {
            status_code: StatusCode::FORBIDDEN,
            body: "wrong signature".to_owned(),
        });
    }

    let (aci_str, timestamp_str) = claims.split_once('.').ok_or(HTTPError {
        status_code: StatusCode::FORBIDDEN,
        body: "failed to split claims".to_owned(),
    })?;

    let aci = ServiceId::parse_from_service_id_string(aci_str).ok_or(HTTPError {
        status_code: StatusCode::FORBIDDEN,
        body: "Failed to parse aci".to_owned(),
    })?;

    let timestamp = timestamp_str.parse().map_err(|_| HTTPError {
        status_code: StatusCode::INTERNAL_SERVER_ERROR,
        body: "Failed to parse timestamp from claims".to_owned(),
    })?;

    let time_then = Duration::from_millis(timestamp);
    let time_now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    let elapsed_time = time_now - time_then;
    if elapsed_time.as_secs() > 600 {
        return Err(HTTPError {
            status_code: StatusCode::FORBIDDEN,
            body: "Too slow to link".to_owned(),
        });
    }

    let accounts = state.accounts.lock().await;
    let aci = service_id_aci(aci)?;
    let account = accounts.get_account(&aci).await.map_err(|e| HTTPError {
        status_code: StatusCode::INTERNAL_SERVER_ERROR,
        body: e.to_string(),
    })?;

    let max_id: u32 = accounts
        .get_devices(&aci)
        .await
        .map_err(|e| HTTPError {
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
            body: e.to_string(),
        })?
        .iter()
        .map(|d| d.device_id.into())
        .max()
        .expect("User has primary device");
    let next_id = max_id + 1;
    drop(accounts);
    let identity_key = account.identity_key;
    let activation_info = link_device_request.device_activation;
    let aci: Aci = Uuid::new_v4().into();
    let hash = SaltedTokenHash::generate_for(auth_header.password())?;
    let device = Device::builder()
        .registration_id(activation_info.registration_id)
        .device_id(next_id.into())
        .name(activation_info.device_name)
        .created(time_now_u128())
        .auth_token(hash.hash())
        .salt(hash.salt())
        .build();

    let protocol_addr = ProtocolAddress::new(aci.service_id_string(), device.device_id);
    add_key_bundle(
        &mut state,
        &identity_key,
        &protocol_addr,
        activation_info.key_bundle,
    )
    .await;
    Ok(LinkDeviceResponse {
        aci: aci.service_id_string(),
        device_id: device.device_id.into(),
    })
    .map(Json)
}

/// Handle device linking
pub async fn delete_device_endpoint(
    State(state): State<ServerState>,
    Path(device_id): Path<u32>,
    authenticated_device: AuthenticatedDevice,
) -> Result<(), HTTPError> {
    let auth_id = authenticated_device.device.device_id;
    if auth_id != 1.into() && auth_id != device_id.into() {
        return Err(HTTPError {
            status_code: StatusCode::UNAUTHORIZED,
            body: "".into(),
        });
    }

    if device_id == 1 {
        return Err(HTTPError {
            status_code: StatusCode::FORBIDDEN,
            body: "".to_owned(),
        });
    }
    state
        .accounts
        .lock()
        .await
        .remove_device(ProtocolAddress::new(
            authenticated_device.account.aci.service_id_string(),
            device_id.into(),
        ))
        .await
        .map_err(|_| HTTPError {
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
            body: "".to_owned(),
        })
}
