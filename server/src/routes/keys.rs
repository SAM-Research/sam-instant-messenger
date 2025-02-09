use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use libsignal_protocol::{IdentityKey, ProtocolAddress, ServiceId};

use crate::{
    auth::{
        authenticate::{service_id_aci, verify_key_signature},
        authenticated_device::AuthenticatedDevice,
    },
    error::HTTPError,
    state::{
        device::Device,
        traits::key_manager::{KeyKind, KeyManager, KeyType},
        ServerState,
    },
    ServerError,
};
use sam_common::api::keys::{BundleResponse, KeyBundle, PublishKeyBundle, PublishKeyBundleRequest};

pub async fn add_key_bundle(
    state: &mut ServerState,
    identity_key: &IdentityKey,
    protocol_addr: &ProtocolAddress,
    key_bundle: PublishKeyBundle,
) -> Result<(), ServerError> {
    let mut key_mgr = state.keys.lock().await;

    if let Some(keys) = key_bundle.pre_keys {
        for key in keys {
            key_mgr.add_key(protocol_addr, KeyType::Normal(key)).await?;
        }
    }

    if let Some(key) = key_bundle.signed_pre_key {
        verify_key_signature(identity_key, &key)?;
        let key = KeyType::NormalSigned(key);
        key_mgr.add_key(&protocol_addr, key).await?;
    }

    if let Some(keys) = key_bundle.pq_pre_keys {
        for key in keys {
            verify_key_signature(identity_key, &key)?;
            let key = KeyType::QuantumSigned(key);
            key_mgr.add_key(protocol_addr, key).await?;
        }
    }

    if let Some(key) = key_bundle.pq_last_resort_pre_key {
        verify_key_signature(identity_key, &key)?;
        let key = KeyType::QuantumSigned(key);
        key_mgr.add_key(&protocol_addr, key).await?;
    }
    Ok(())
}

pub async fn get_key_bundle(
    keys: &mut dyn KeyManager,
    device: &Device,
    protocol_addr: &ProtocolAddress,
) -> Result<KeyBundle, ServerError> {
    let pq_pre_key = keys
        .get_key(protocol_addr, KeyKind::QuantumSigned)
        .await
        .map(|k| k.unwrap_signed())?;
    let signed_pre_key = keys
        .get_key(protocol_addr, KeyKind::NormalSigned)
        .await
        .map(|k| k.unwrap_signed())?;

    let pre_key = keys
        .get_key(protocol_addr, KeyKind::Normal)
        .await
        .map(|k| k.unwrap())
        .ok();

    Ok(KeyBundle {
        device_id: device.device_id.into(),
        registration_id: device.registration_id,
        pre_key: pre_key,
        pq_pre_key: pq_pre_key,
        signed_pre_key: signed_pre_key,
    })
}

/// Returns key bundles for users devices
pub async fn keys_bundles_endpoint(
    Path(account_id): Path<String>,
    State(state): State<ServerState>,
) -> Result<Json<BundleResponse>, HTTPError> {
    let service_id = ServiceId::parse_from_service_id_string(&account_id).ok_or(HTTPError {
        status_code: StatusCode::INTERNAL_SERVER_ERROR,
        body: "Error parsing service id for target user".into(),
    })?;
    let accounts = state.accounts.lock().await;

    let aci = service_id_aci(service_id)?;
    let identity_key = accounts
        .get_account(&aci)
        .await
        .map_err(|e| HTTPError {
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
            body: e.to_string(),
        })?
        .identity_key;

    let devices = accounts.get_devices(&aci).await.map_err(|e| HTTPError {
        status_code: StatusCode::INTERNAL_SERVER_ERROR,
        body: e.to_string(),
    })?;

    let mut keys_guard = state.keys.lock().await;
    let keys = keys_guard.as_mut();

    let mut bundles = vec![];
    for device in devices {
        let addr = ProtocolAddress::new(aci.service_id_string(), device.device_id);
        let bundle = get_key_bundle(keys, &device, &addr)
            .await
            .map_err(|e| HTTPError {
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
                body: e.to_string(),
            })?;
        bundles.push(bundle);
    }

    Ok(BundleResponse {
        identity_key,
        devices: bundles,
    })
    .map(Json)
}

/// Handle publish of new key bundles
#[axum::debug_handler]
pub async fn publish_keys_endpoint(
    State(mut state): State<ServerState>,
    authenticated_device: AuthenticatedDevice,
    Json(key_bundle): Json<PublishKeyBundleRequest>,
) -> Result<(), HTTPError> {
    add_key_bundle(
        &mut state,
        &authenticated_device.account.identity_key,
        &authenticated_device.protocol_address(),
        key_bundle,
    )
    .await
    .map_err(|e| HTTPError {
        status_code: StatusCode::INTERNAL_SERVER_ERROR,
        body: e.to_string(),
    })
}
