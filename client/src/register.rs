use base64::{prelude::BASE64_STANDARD, Engine as _};
use libsignal_protocol::IdentityKeyPair;
use rand::{CryptoRng, Rng};
use sam_common::{
    api::{device::DeviceActivationInfo, RegistrationRequest},
    RegistrationId,
};

use crate::{
    keygen::KeyManager as _,
    net::SignalApiClient,
    storage::{AccountStore, StoreConfig},
    Client, ClientError,
};

const PASSWORD_LENGTH: usize = 16;

fn generate_password<R: Rng>(rng: &mut R) -> String {
    let mut password = [0u8; PASSWORD_LENGTH];
    rng.fill(&mut password);
    let password = BASE64_STANDARD.encode(password);
    password[0..password.len() - 2].to_owned()
}

pub async fn register_client<
    S: StoreConfig,
    H: SignalApiClient<Error = impl Into<ClientError>>,
    R: CryptoRng + Rng,
>(
    storage_config: S,
    http_client: H,
    mut csprng: &mut R,
) -> Result<Client<S::StoreType>, ClientError> {
    let name = "DeviceName".to_owned();
    let registration_id = RegistrationId::generate(&mut csprng);
    let id_key_pair = IdentityKeyPair::generate(&mut csprng);

    let mut store = storage_config
        .create_store(id_key_pair, registration_id)
        .await?;
    let signed_pk = store.generate_signed_pre_key(&mut csprng).await?;
    let pq_last_resort = store.generate_kyber_pre_key().await?;

    let password = generate_password(&mut csprng);

    let key_bundle = store
        .generate_one_time_pre_keys(&mut csprng)
        .await?
        .to_bundle(signed_pk, pq_last_resort);

    let registration_request = RegistrationRequest {
        identity_key: id_key_pair.identity_key().to_owned(),
        device_activation: DeviceActivationInfo {
            name,
            registration_id,
            key_bundle,
        },
    };

    let _response = http_client
        .register_client(password.to_owned(), registration_request)
        .await
        .map_err(Into::into)?;

    //let aci = response.account_id.into();

    //store.account_store.set_aci(aci).await?;
    store.account_store.set_password(password).await?;

    Ok(Client { store })
}
