use bon::bon;
use libsignal_protocol::IdentityKeyPair;
use log::error;
use rand::rngs::OsRng;
use sam_common::{
    address::RegistrationId,
    api::{
        device::DeviceActivationInfo, keys::RegistrationPreKeys, LinkDeviceToken,
        RegistrationRequest,
    },
    sam_message::ServerEnvelope,
    AccountId, DeviceId,
};
use tokio::sync::broadcast::Receiver;
use tokio::sync::mpsc::Receiver as MpscReceiver;

use crate::{
    encryption::{
        encrypt::{decrypt, encrypt},
        envelope::DecryptedEnvelope,
        password::generate_password,
    },
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
        traits::message::MessageStore,
        AccountStore, Store, StoreConfig, StoreType,
    },
    ClientError,
};

pub struct Client<T: StoreType, U: ApiClient, V: SamProtocolClient> {
    store: Store<T>,
    api_client: U,
    protocol_client: V,
    envelope_queue: MpscReceiver<ServerEnvelope>,
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

        let mut protocol_client = protocol_config
            .create(account_id, 1.into(), password.clone())
            .await?;

        store
            .account_store
            .set_username(username.to_owned())
            .await?;
        store.account_store.set_account_id(account_id).await?;
        store.account_store.set_device_id(1.into()).await?;
        store.account_store.set_password(password).await?;

        let queue = protocol_client.connect().await?;

        Ok(Self {
            store,
            protocol_client,
            api_client,
            envelope_queue: queue,
        })
    }

    /// Instantiate a client from a valid store
    #[builder]
    pub async fn from_store(
        store: Store<T>,
        protocol_config: impl ProtocolConfig<ProtocolClient = V>,
        api_client_config: impl ApiClientConfig<ApiClient = U>,
    ) -> Result<Self, ClientError> {
        let account_id = store.account_store.get_account_id().await?;
        let device_id = store.account_store.get_device_id().await?;
        let password = store.account_store.get_password().await?;
        let mut protocol_client = protocol_config
            .create(account_id, device_id, password)
            .await?;
        let queue = protocol_client.connect().await?;
        Ok(Self {
            store,
            protocol_client,
            api_client: api_client_config.create().await?,
            envelope_queue: queue,
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
    pub async fn get_user_account_id(&self, _username: &str) -> Result<AccountId, ClientError> {
        todo!()
    }

    /// Send any message to recipient
    /// Should also send to users other devices
    pub async fn send_message(
        &mut self,
        recipient: AccountId,
        msg: impl Into<Vec<u8>>,
    ) -> Result<bool, ClientError> {
        let envelope = encrypt(msg, recipient, &mut self.store).await?;
        self.protocol_client
            .send_message(envelope)
            .await
            .map_err(ClientError::from)
    }

    /// Returns a broadcast receiver for incoming messages that have been decrypted
    pub async fn subscribe(&mut self) -> Receiver<DecryptedEnvelope> {
        self.store.message_store.subscribe()
    }

    pub async fn process_messages(&mut self) -> Result<(), ClientError> {
        while let Some(envelope) = self.envelope_queue.recv().await {
            // TODO: How should we handle failure to decrypt and/or store message?
            let envelope = match decrypt(envelope, &mut self.store).await {
                Ok(denvelope) => denvelope,
                Err(e) => {
                    error!("Failed to decrypt message {e}");
                    continue;
                }
            };

            let _ = self
                .store
                .message_store
                .store_message(envelope)
                .await
                .inspect_err(|e| error!("Failed to store message {e}"));
        }
        Ok(())
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
