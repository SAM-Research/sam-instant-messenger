use std::time::SystemTime;

use libsignal_core::ProtocolAddress;
use libsignal_protocol::{process_prekey_bundle, IdentityKeyStore};
use log::{debug, error};
use rand::{CryptoRng, Rng};
use sam_common::{
    api::{
        device::DeviceActivationInfo, LinkDeviceRequest, LinkDeviceToken, PublishPreKeys,
        RegistrationRequest,
    },
    sam_message::ServerEnvelope,
    AccountId, DeviceId,
};
use tokio::sync::mpsc::Receiver;

use crate::{
    encryption::{decrypt, encrypt, generate_password},
    net::{
        protocol::{MessageStatus, SamProtocolClient},
        ApiClient,
    },
    storage::{
        key_generation::{
            create_registration_pre_keys, generate_ec_pre_keys, generate_pq_pre_keys,
            into_libsignal_bundle, KyberKeyGenerator, SignedPreKeyGenerator,
        },
        AccountStore, ContactStore, MessageStore, Store, StoreType,
    },
    ClientError,
};

pub async fn provision_device<T: StoreType, R: Rng + CryptoRng>(
    api_client: &impl ApiClient,
    store: &mut Store<T>,
    device_name: &str,
    token: LinkDeviceToken,
    upload_prekey_count: usize,
    password_length: usize,
    mut rng: &mut R,
) -> Result<(), ClientError> {
    let id_key_pair = store.identity_key_store.get_identity_key_pair().await?;
    let key_bundle =
        create_registration_pre_keys(store, upload_prekey_count, id_key_pair, &mut rng).await?;
    let request = LinkDeviceRequest {
        token,
        device_activation: DeviceActivationInfo {
            name: device_name.to_owned(),
            registration_id: store
                .identity_key_store
                .get_local_registration_id()
                .await?
                .into(),
            key_bundle,
        },
    };
    let password = generate_password(password_length, &mut rng);
    let response = api_client.link_device(&password, request).await?;
    store.account_store.set_username(response.username).await?;
    store
        .account_store
        .set_account_id(response.account_id)
        .await?;
    store
        .account_store
        .set_device_id(response.device_id)
        .await?;
    store.account_store.set_password(password.clone()).await?;
    store
        .contact_store
        .add_device(response.account_id, response.device_id)
        .await
}

pub async fn register_account<T: StoreType, R: Rng + CryptoRng>(
    api_client: &impl ApiClient,
    store: &mut Store<T>,
    username: &str,
    device_name: &str,
    password_length: usize,
    upload_prekey_count: usize,
    mut rng: &mut R,
) -> Result<(), ClientError> {
    let password = generate_password(password_length, &mut rng);
    let id_pair = store.identity_key_store.get_identity_key_pair().await?;
    let key_bundle =
        create_registration_pre_keys(store, upload_prekey_count, id_pair, &mut rng).await?;
    let registration_request = RegistrationRequest {
        identity_key: id_pair.identity_key().to_owned(),
        device_activation: DeviceActivationInfo {
            name: device_name.to_owned(),
            registration_id: store
                .identity_key_store
                .get_local_registration_id()
                .await?
                .into(),
            key_bundle,
        },
    };

    let account_id = api_client
        .register_account(username, &password, registration_request)
        .await?
        .account_id;
    store
        .account_store
        .set_username(username.to_owned())
        .await?;
    let device_id = 1.into();
    store.account_store.set_account_id(account_id).await?;
    store.account_store.set_device_id(device_id).await?;
    store.account_store.set_password(password).await?;
    store.contact_store.add_device(account_id, device_id).await
}

pub async fn process_messages<T: StoreType>(
    store: &mut Store<T>,
    envelope_queue: &mut Receiver<ServerEnvelope>,
    block: bool,
) -> Result<(), ClientError> {
    if !block && envelope_queue.is_empty() {
        return Ok(());
    }
    while let Some(envelope) = envelope_queue.recv().await {
        // TODO: How should we handle failure to decrypt and/or store message?
        let envelope = match decrypt(envelope, store).await {
            Ok(denvelope) => denvelope,
            Err(e) => {
                error!("Failed to decrypt message: {e}");
                break;
            }
        };

        store
            .contact_store
            .add_device(envelope.source_account_id(), envelope.source_device_id())
            .await?;

        let _ = store
            .message_store
            .store_message(envelope)
            .await
            .inspect_err(|e| error!("Failed to store message {e}"));
        if envelope_queue.is_empty() {
            break;
        }
    }
    Ok(())
}

pub async fn send_message<T: StoreType, R: Rng + CryptoRng>(
    store: &mut Store<T>,
    api_client: &impl ApiClient,
    ws_client: &mut impl SamProtocolClient,
    recipient: AccountId,
    msg: impl Into<Vec<u8>>,
    mut rng: &mut R,
) -> Result<(), ClientError> {
    if !store.contact_store.contains_contact(recipient).await? {
        fetch_prekeys(store, api_client, recipient, None, &mut rng).await?;
    }

    let my_id = store.account_store.get_account_id().await?;
    if !store.contact_store.contains_contact(my_id).await? {
        fetch_prekeys(store, api_client, my_id, None, &mut rng).await?;
    }
    let envelope = encrypt(msg, vec![recipient, my_id], store).await?;
    let status = ws_client.send_message(envelope).await?;
    match status {
        MessageStatus::ExtraDevices(device_lists) => {
            for list in device_lists {
                for device in list.devices {
                    store
                        .contact_store
                        .remove_device(list.account_id, device)
                        .await?;
                }
            }
            Ok(())
        }
        MessageStatus::MissingDevices(device_lists) => {
            for list in device_lists {
                fetch_prekeys(
                    store,
                    api_client,
                    list.account_id,
                    Some(list.devices),
                    &mut rng,
                )
                .await?;
            }
            Err(ClientError::MissingDevices)
        }
        MessageStatus::Ok => Ok(()),
    }
}

pub async fn fetch_prekeys<T: StoreType, R: Rng + CryptoRng>(
    store: &mut Store<T>,
    api_client: &impl ApiClient,
    account_id: AccountId,
    devices: Option<Vec<DeviceId>>,
    mut rng: &mut R,
) -> Result<(), ClientError> {
    let prekey_bundles = api_client
        .get_pre_key_bundles(
            store.account_store.get_account_id().await?,
            store.account_store.get_device_id().await?,
            store.account_store.get_password().await?.as_str(),
            account_id,
            devices,
        )
        .await?;
    let time = SystemTime::now();
    for bundle in prekey_bundles.bundles {
        let device_id = bundle.device_id;
        store
            .contact_store
            .add_device(account_id, device_id.into())
            .await?;
        let libsignal_bundle = into_libsignal_bundle(bundle, prekey_bundles.identity_key)?;
        process_prekey_bundle(
            &ProtocolAddress::new(account_id.to_string(), device_id.into()),
            &mut store.session_store,
            &mut store.identity_key_store,
            &libsignal_bundle,
            time,
            &mut rng,
        )
        .await
        .inspect_err(|e| debug!("{e}"))
        .map_err(|_| ClientError::FailedToProcessPrekeyBundle)?;
    }
    Ok(())
}

pub async fn publish_prekeys<T: StoreType, R: Rng + CryptoRng>(
    store: &mut Store<T>,
    api_client: &impl ApiClient,
    onetime_prekeys: usize,
    new_signed_prekey: bool,
    new_last_resort: bool,
    mut rng: &mut R,
) -> Result<(), ClientError> {
    let id_pair = store.identity_key_store.get_identity_key_pair().await?;
    let onetime_ec_prekeys =
        generate_ec_pre_keys(&mut store.pre_key_store, onetime_prekeys, &mut rng).await?;
    let onetime_pq_prekeys = generate_pq_pre_keys(
        id_pair.private_key(),
        &mut store.kyber_pre_key_store,
        onetime_prekeys,
    )
    .await?;

    Ok(api_client
        .publish_pre_keys(
            store.account_store.get_account_id().await?,
            store.account_store.get_device_id().await?,
            store.account_store.get_password().await?.as_str(),
            PublishPreKeys {
                pre_keys: Some(onetime_ec_prekeys),
                signed_pre_key: new_signed_prekey.then_some(
                    store
                        .signed_pre_key_store
                        .generate_key(&mut rng, id_pair.private_key())
                        .await?
                        .into(),
                ),
                pq_pre_keys: Some(onetime_pq_prekeys),
                pq_last_resort_pre_key: new_last_resort.then_some(
                    store
                        .kyber_pre_key_store
                        .generate_key(id_pair.private_key())
                        .await?
                        .into(),
                ),
            },
        )
        .await?)
}
