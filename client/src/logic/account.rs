use libsignal_protocol::IdentityKeyStore;
use rand::{CryptoRng, Rng};
use sam_common::api::{device::DeviceActivationInfo, RegistrationRequest};

use crate::{
    encryption::generate_password,
    net::ApiClient,
    storage::{
        key_generation::create_registration_pre_keys, AccountStore, ContactStore, Store, StoreType,
    },
};

use super::LogicError;

pub async fn register_account<T: StoreType, R: Rng + CryptoRng>(
    api_client: &impl ApiClient,
    store: &mut Store<T>,
    username: &str,
    device_name: &str,
    password_length: usize,
    upload_prekey_count: usize,
    mut rng: &mut R,
) -> Result<(), LogicError> {
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
    store
        .contact_store
        .add_device(account_id, device_id)
        .await?;
    Ok(())
}
