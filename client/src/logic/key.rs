use std::time::SystemTime;

use libsignal_core::ProtocolAddress;
use libsignal_protocol::{process_prekey_bundle, IdentityKeyStore};
use log::debug;
use rand::{CryptoRng, Rng};
use sam_common::{api::PublishPreKeys, AccountId, DeviceId};

use crate::{
    net::ApiClient,
    storage::{
        key_generation::{
            generate_ec_pre_keys, generate_pq_pre_keys, into_libsignal_bundle, KyberKeyGenerator,
            SignedPreKeyGenerator,
        },
        AccountStore, ContactStore, Store, StoreType,
    },
};

use super::LogicError;

pub async fn fetch_prekeys<T: StoreType, R: Rng + CryptoRng>(
    store: &mut Store<T>,
    api_client: &impl ApiClient,
    account_id: AccountId,
    devices: Option<Vec<DeviceId>>,
    mut rng: &mut R,
) -> Result<(), LogicError> {
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
        .map_err(|_| LogicError::FailedToProcessPrekeyBundle)?;
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
) -> Result<(), LogicError> {
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
