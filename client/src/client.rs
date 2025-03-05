use base64::{prelude::BASE64_STANDARD, Engine as _};
use bon::bon;
use libsignal_protocol::IdentityKeyPair;
use rand::{rngs::OsRng, Rng};
use sam_common::{
    address::RegistrationId,
    api::{
        device::DeviceActivationInfo, keys::RegistrationPreKeys, LinkDeviceToken,
        RegistrationRequest,
    },
    AccountId, DeviceId,
};
use tokio::sync::broadcast::Receiver;

use crate::{
    encryption::envelope::DecryptedEnvelope,
    net::{
        api_trait::ApiClientConfig,
        protocol::traits::{ProtocolConfig, SamProtocolClient},
        ApiClient,
    },
    storage::{
        key_generation::{
            generate_ec_pre_keys, generate_pq_pre_keys, KyberKeyGenerator as _,
            SignedPreKeyGenerator as _,
        },
        AccountStore, Store, StoreConfig, StoreType,
    },
    ClientError,
};

pub struct Client<T: StoreType, U: ApiClient, V: SamProtocolClient> {
    store: Store<T>,
    _api_client: U,
    _protocol_client: V,
}

const PASSWORD_LENGTH: usize = 16;

fn generate_password<R: Rng>(rng: &mut R) -> String {
    let mut password = [0u8; PASSWORD_LENGTH];
    rng.fill(&mut password);
    let password = BASE64_STANDARD.encode(password);
    password[0..password.len() - 2].to_owned()
}

#[bon]
impl<T: StoreType, U: ApiClient, V: SamProtocolClient> Client<T, U, V> {
    /// Creates a new client for the account described in the token
    #[builder]
    pub async fn from_provisioning(
        _store_config: impl StoreConfig<StoreType = T>,
        _protocol_config: impl ProtocolConfig,
        _api_client_config: impl ApiClientConfig,
        _device_name: &str,
        _password: &str,
        _token: LinkDeviceToken,
    ) -> Result<Self, ClientError> {
        todo!()
    }

    /// Register a new account from a clean store
    #[builder]
    pub async fn from_registration(
        store_config: impl StoreConfig<StoreType = T>,
        protocol_config: impl ProtocolConfig<ProtocolClient = V>,
        api_client_config: impl ApiClientConfig<ApiClient = U>,
        username: &str,
        device_name: &str,
    ) -> Result<Self, ClientError> {
        let mut csprng = OsRng;
        let registration_id = RegistrationId::generate(&mut csprng);
        let id_key_pair = IdentityKeyPair::generate(&mut csprng);
        let mut store = store_config
            .create_store(id_key_pair, registration_id)
            .await?;

        let password = generate_password(&mut csprng);
        let key_bundle = RegistrationPreKeys {
            pre_keys: Some(generate_ec_pre_keys(&mut store.pre_key_store, 100, &mut csprng).await?),
            signed_pre_key: store
                .signed_pre_key_store
                .generate_key(&mut csprng, id_key_pair.private_key())
                .await?
                .into(),
            pq_pre_keys: Some(
                generate_pq_pre_keys(
                    id_key_pair.private_key(),
                    &mut store.kyber_pre_key_store,
                    100,
                )
                .await?,
            ),
            pq_last_resort_pre_key: store
                .kyber_pre_key_store
                .generate_key(id_key_pair.private_key())
                .await?
                .into(),
        };

        let registration_request = RegistrationRequest {
            identity_key: id_key_pair.identity_key().to_owned(),
            device_activation: DeviceActivationInfo {
                name: device_name.to_owned(),
                registration_id,
                key_bundle,
            },
        };

        let api_client = api_client_config.create().await?;

        let response = api_client
            .register_account(username, &password, registration_request)
            .await?;

        let account_id = response.account_id.into();

        let protocol_client = protocol_config.create().await?;

        store
            .account_store
            .set_username(username.to_owned())
            .await?;
        store
            .account_store
            .set_username(username.to_owned())
            .await?;
        store.account_store.set_account_id(account_id).await?;
        //store.account_store.set_device_id(1.into()).await?;
        store.account_store.set_password(password).await?;

        Ok(Client {
            store,
            _protocol_client: protocol_client,
            _api_client: api_client,
        })
    }

    /// Instantiate a client from a valid store
    #[builder]
    pub async fn from_store(
        store: Store<T>,
        protocol_config: impl ProtocolConfig<ProtocolClient = V>,
        api_client_config: impl ApiClientConfig<ApiClient = U>,
    ) -> Result<Self, ClientError> {
        Ok(Self {
            store,
            _protocol_client: protocol_config.create().await?,
            _api_client: api_client_config.create().await?,
        })
    }

    pub async fn account_id(&self) -> Result<AccountId, ClientError> {
        self.store.account_store.get_account_id().await
    }

    /// Delete Account and consume client
    pub async fn delete_account(self) {
        todo!()
    }

    /// Delete device and consume client
    pub async fn delete_device(self) {
        todo!()
    }

    /// Get users account id from username
    pub async fn get_user_account_id(&self, _username: &str) -> Result<AccountId, ClientError> {
        todo!()
    }

    /// Send any message to receipient
    /// Should also send to users other devices
    pub async fn send_message(
        &mut self,
        _receipient: AccountId,
        _msg: impl Into<Vec<u8>>,
    ) -> Result<bool, ClientError> {
        todo!()
    }

    /// Returns a broadcast receiver for incoming messages that have been decrypted
    pub async fn subscribe(&mut self) -> Result<Receiver<DecryptedEnvelope>, ClientError> {
        todo!()
    }

    /// publish ec, pq, last resort or last resort of amount
    #[builder]
    pub async fn publish_prekeys(
        &mut self,
        #[builder(default)] _onetime_ec_keys: u32,
        #[builder(default)] _onetime_pq_prekeys: u32,
        #[builder(default = false)] _new_signed_prekey: bool,
        #[builder(default = false)] _new_last_resort: bool,
    ) -> Result<(), ClientError> {
        todo!()
    }

    /// Fetch key bundles for account_id
    pub async fn fetch_prekeys(
        &mut self,
        _account_id: AccountId,
        _devices: Vec<DeviceId>,
    ) -> Result<(), ClientError> {
        todo!()
    }

    /// Create a provision token to be used on another client to activate
    pub async fn create_provision(&mut self) -> Result<LinkDeviceToken, ClientError> {
        todo!()
    }
}
