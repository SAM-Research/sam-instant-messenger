use libsignal_protocol::IdentityKeyStore;
use rand::{CryptoRng, Rng};
use sam_common::api::{device::DeviceActivationInfo, LinkDeviceRequest, LinkDeviceToken};

use crate::{
    encryption::generate_password,
    net::ApiClient,
    storage::{
        key_generation::create_registration_pre_keys, AccountStore, ContactStore, SamStoreType,
        Store,
    },
    ClientError,
};

pub async fn provision_device<T: SamStoreType, R: Rng + CryptoRng>(
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
