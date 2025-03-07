use crate::{
    encryption::{envelope::DecryptedEnvelope, password::generate_password},
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
use bon::bon;
use libsignal_protocol::{IdentityKeyPair, IdentityKeyStore};
use rand::rngs::OsRng;
use sam_common::api::keys::PreKeyBundles;
use sam_common::api::{PqPreKey, PublishPreKeys, SignedEcPreKey};
use sam_common::{
    address::RegistrationId,
    api::{
        device::DeviceActivationInfo, keys::RegistrationPreKeys, LinkDeviceToken,
        RegistrationRequest,
    },
    AccountId, DeviceId,
};
use tokio::sync::broadcast::Receiver;

pub struct Client<T: StoreType, U: ApiClient, V: SamProtocolClient> {
    store: Store<T>,
    api_client: U,
    _protocol_client: V,
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
        #[builder(default = 100)] upload_prekey_count: usize,
        #[builder(default = 16)] password_length: usize,
    ) -> Result<Self, ClientError> {
        let mut csprng = OsRng;
        let registration_id = RegistrationId::generate(&mut csprng);
        let id_key_pair = IdentityKeyPair::generate(&mut csprng);
        let mut store = store_config
            .create_store(id_key_pair, registration_id)
            .await?;

        let password = generate_password(password_length, &mut csprng);
        let key_bundle = RegistrationPreKeys {
            pre_keys: Some(
                generate_ec_pre_keys(&mut store.pre_key_store, upload_prekey_count, &mut csprng)
                    .await?,
            ),
            signed_pre_key: store
                .signed_pre_key_store
                .generate_key(&mut csprng, id_key_pair.private_key())
                .await?
                .into(),
            pq_pre_keys: Some(
                generate_pq_pre_keys(
                    id_key_pair.private_key(),
                    &mut store.kyber_pre_key_store,
                    upload_prekey_count,
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

        let account_id = response.account_id;

        let protocol_client = protocol_config.create().await?;

        store
            .account_store
            .set_username(username.to_owned())
            .await?;
        store.account_store.set_account_id(account_id).await?;
        store.account_store.set_device_id(1.into()).await?;
        store.account_store.set_password(password).await?;

        Ok(Client {
            store,
            _protocol_client: protocol_client,
            api_client,
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
            api_client: api_client_config.create().await?,
        })
    }

    pub async fn account_id(&self) -> Result<AccountId, ClientError> {
        self.store.account_store.get_account_id().await
    }

    async fn device_id(&self) -> Result<DeviceId, ClientError> {
        self.store.account_store.get_device_id().await
    }

    /// Delete Account and consume client
    pub async fn delete_account(self) -> Result<(), (Self, ClientError)> {
        let account_id = self.account_id().await;
        let device_id = self.device_id().await;
        let password = self.store.account_store.get_password().await;

        let Ok(account_id) = account_id else {
            return Err((self, account_id.unwrap_err()));
        };

        let Ok(device_id) = device_id else {
            return Err((self, device_id.unwrap_err()));
        };

        let Ok(password) = password else {
            return Err((self, password.unwrap_err()));
        };

        let delete_result = self
            .api_client
            .delete_account(account_id, device_id, &password)
            .await;

        let Ok(()) = delete_result else {
            return Err((self, ClientError::Api(delete_result.unwrap_err())));
        };

        Ok(())
    }

    /// Delete device and consume client
    pub async fn delete_device(self) -> Result<(), (Self, ClientError)> {
        todo!()
    }

    /// Get users account id from username
    pub async fn get_user_account_id(&self, username: &str) -> Result<AccountId, ClientError> {
        let account_id = self
            .api_client
            .get_user_account_id(
                self.account_id().await?,
                self.store.account_store.get_device_id().await?,
                self.store.account_store.get_password().await?.as_str(),
                username,
            )
            .await?;

        Ok(account_id)
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
        #[builder(default)] onetime_prekeys: usize,
        #[builder(default = false)] new_signed_prekey: bool,
        #[builder(default = false)] new_last_resort: bool,
    ) -> Result<(), ClientError> {
        let mut csprng = OsRng;
        let id_key_pair = self
            .store
            .identity_key_store
            .get_identity_key_pair()
            .await?;
        let onetime_ec_prekeys =
            generate_ec_pre_keys(&mut self.store.pre_key_store, onetime_prekeys, &mut csprng)
                .await?;
        let onetime_pq_prekeys = generate_pq_pre_keys(
            id_key_pair.private_key(),
            &mut self.store.kyber_pre_key_store,
            onetime_prekeys,
        )
        .await?;

        let signed_pre_key: Option<SignedEcPreKey> = match new_signed_prekey {
            true => Some(
                self.store
                    .signed_pre_key_store
                    .generate_key(&mut csprng, id_key_pair.private_key())
                    .await?
                    .into(),
            ),
            false => None,
        };

        let last_resort_key: Option<PqPreKey> = match new_last_resort {
            true => Some(
                self.store
                    .kyber_pre_key_store
                    .generate_key(id_key_pair.private_key())
                    .await?
                    .into(),
            ),
            false => None,
        };

        let pre_key_bundle = PublishPreKeys {
            pre_keys: Some(onetime_ec_prekeys),
            signed_pre_key,
            pq_pre_keys: Some(onetime_pq_prekeys),
            pq_last_resort_pre_key: last_resort_key,
        };

        self.api_client
            .publish_pre_keys(
                self.account_id().await?,
                self.store.account_store.get_device_id().await?,
                self.store.account_store.get_password().await?.as_str(),
                pre_key_bundle,
            )
            .await?;

        Ok(())
    }

    /// Fetch key bundles for account_id
    pub async fn fetch_prekeys_for_specific_devices(
        &mut self,
        account_id: AccountId,
        devices: Vec<DeviceId>,
    ) -> Result<PreKeyBundles, ClientError> {
        let prekey_bundles = self
            .api_client
            .get_pre_keys_for_specific_devices(
                self.account_id().await?,
                self.store.account_store.get_device_id().await?,
                self.store.account_store.get_password().await?.as_str(),
                account_id,
                devices,
            )
            .await?;

        Ok(prekey_bundles)
    }

    pub async fn fetch_prekeys_for_all_devices(
        &mut self,
        account_id: AccountId,
    ) -> Result<PreKeyBundles, ClientError> {
        let prekey_bundles = self
            .api_client
            .get_pre_keys_for_all_devices(
                self.account_id().await?,
                self.store.account_store.get_device_id().await?,
                self.store.account_store.get_password().await?.as_str(),
                account_id,
            )
            .await?;

        Ok(prekey_bundles)
    }

    /// Create a provision token to be used on another client to activate
    pub async fn create_provision(&mut self) -> Result<LinkDeviceToken, ClientError> {
        todo!()
    }
}
