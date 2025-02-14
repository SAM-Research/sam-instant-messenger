use libsignal_protocol::IdentityKey;
use sam_common::api::keys::{Key, KeyBundle, KeyBundleResponse, PublishKeyBundle};
use uuid::Uuid;

use crate::{
    managers::traits::{
        account_manager::AccountManager,
        device_manager::DeviceManager,
        key_manager::{LastResortKeyManager, PqPreKeyManager, PreKeyManager, SignedPreKeyManager},
    },
    state::{state_type::StateType, ServerState},
    ServerError,
};

pub async fn get_keybundle<T: StateType>(
    state: &ServerState<T>,
    account_id: &Uuid,
    registration_id: &u32,
    device_id: &u32,
) -> Result<KeyBundle, ServerError> {
    let mut keys = state.keys.lock().await;

    let pre_key = keys.get_pre_key(account_id, device_id).await?; // TODO: might need some inspection to see if its a not found or actual error
    let pq_pre_key = keys.get_pq_pre_key(account_id, device_id).await?;
    let signed_pre_key = keys.get_signed_pre_key(account_id, device_id).await?;

    let pre_key = match pre_key {
        Some(key) => {
            keys.remove_pre_key(account_id, device_id, key.id()).await?;
            Some(key)
        }
        None => None,
    };

    let pq_pre_key = match pq_pre_key {
        Some(key) => {
            keys.remove_pq_pre_key(account_id, device_id, key.id())
                .await?;
            key
        }
        None => keys.get_last_resort_key(account_id, device_id).await?,
    };

    Ok(KeyBundle {
        device_id: *device_id,
        registration_id: *registration_id,
        pre_key,
        pq_pre_key,
        signed_pre_key,
    })
}

pub async fn add_keybundle<T: StateType>(
    state: &ServerState<T>,
    identity: &IdentityKey,
    account_id: &uuid::Uuid,
    device_id: &u32,
    key_bundle: PublishKeyBundle,
) -> Result<(), ServerError> {
    let mut keys = state.keys.lock().await;
    if let Some(pre_keys) = key_bundle.pre_keys {
        for pre_key in pre_keys {
            keys.add_pre_key(account_id, device_id, pre_key).await?;
        }
    }

    if let Some(key) = key_bundle.signed_pre_key {
        keys.set_signed_pre_key(account_id, device_id, identity, key)
            .await?;
    }

    if let Some(pre_keys) = key_bundle.pq_pre_keys {
        for pre_key in pre_keys {
            keys.add_pq_pre_key(account_id, device_id, identity, pre_key)
                .await?;
        }
    }

    if let Some(key) = key_bundle.pq_last_resort_pre_key {
        keys.set_last_resort_key(account_id, device_id, identity, key)
            .await?
    }
    Ok(())
}

pub async fn get_keybundles<T: StateType>(
    state: &ServerState<T>,
    account_id: &Uuid,
) -> Result<KeyBundleResponse, ServerError> {
    let identity_key = {
        *state
            .accounts
            .lock()
            .await
            .get_account(account_id)
            .await?
            .identity()
    };

    let devices = {
        let devices = state.devices.lock().await;
        let mut device_vec = vec![];
        for id in devices.get_devices(account_id).await? {
            let device = devices.get_device(account_id, &id).await?;
            device_vec.push(device);
        }
        device_vec
    };

    let bundles = {
        let mut bundle_vec = vec![];
        for device in devices {
            bundle_vec.push(
                get_keybundle(state, account_id, &device.registration_id(), &device.id()).await?,
            );
        }
        bundle_vec
    };

    Ok(KeyBundleResponse {
        identity_key,
        bundles,
    })
}

pub async fn publish_keybundle<T: StateType>(
    state: &ServerState<T>,
    account_id: &Uuid,
    device_id: &u32,
    bundle: PublishKeyBundle,
) -> Result<(), ServerError> {
    let identity = {
        *state
            .accounts
            .lock()
            .await
            .get_account(account_id)
            .await?
            .identity()
    };

    add_keybundle(state, &identity, account_id, device_id, bundle).await
}
