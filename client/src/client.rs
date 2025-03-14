use bon::bon;
use libsignal_core::ProtocolAddress;
use libsignal_protocol::{process_prekey_bundle, IdentityKeyPair, IdentityKeyStore};
use log::error;
use rand::rngs::OsRng;
use sam_common::{
    address::RegistrationId,
    api::{
        device::DeviceActivationInfo, keys::RegistrationPreKeys, LinkDeviceRequest,
        LinkDeviceToken, PqPreKey, PublishPreKeys, RegistrationRequest, SignedEcPreKey,
    },
    sam_message::ServerEnvelope,
    AccountId, DeviceId,
};
use std::time::SystemTime;
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
        protocol::traits::{MessageStatus, ProtocolConfig, SamProtocolClient},
        ApiClient,
    },
    storage::{
        key_generation::{
            generate_ec_pre_keys, generate_pq_pre_keys, into_libsignal_bundle,
            KyberKeyGenerator as _, SignedPreKeyGenerator as _,
        },
        traits::message::MessageStore,
        AccountStore, ContactStore, Store, StoreConfig, StoreType,
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
        store_config: impl StoreConfig<StoreType = T>,
        protocol_config: impl ProtocolConfig<ProtocolClient = V>,
        api_client_config: impl ApiClientConfig<ApiClient = U>,
        device_name: &str,
        id_key_pair: IdentityKeyPair,
        token: LinkDeviceToken,
        #[builder(default = 100)] upload_prekey_count: usize,
        #[builder(default = 16)] password_length: usize,
    ) -> Result<Self, ClientError> {
        let mut csprng = OsRng;
        let api_client = api_client_config.create().await?;
        let registration_id = RegistrationId::generate(&mut csprng);

        let mut store = store_config
            .create_store(id_key_pair, registration_id)
            .await?;

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

        let request = LinkDeviceRequest {
            token,
            device_activation: DeviceActivationInfo {
                name: device_name.to_owned(),
                registration_id: RegistrationId::generate(&mut csprng),
                key_bundle,
            },
        };
        let password = generate_password(password_length, &mut csprng);
        let response = api_client
            .link_device(device_name, &password, request)
            .await?;

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

        let mut protocol_client = protocol_config
            .create(response.account_id, response.device_id, password.clone())
            .await?;

        let queue = protocol_client.connect().await?;

        Ok(Self {
            store,
            api_client,
            protocol_client,
            envelope_queue: queue,
        })
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

    pub async fn device_id(&self) -> Result<DeviceId, ClientError> {
        self.store.account_store.get_device_id().await
    }

    pub async fn identity_key_pair(&self) -> Result<IdentityKeyPair, ClientError> {
        Ok(self
            .store
            .identity_key_store
            .get_identity_key_pair()
            .await?)
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

    /// Delete this device and consume client.
    /// This cannot be done for the primary device. See `unlink_device` if you want to delete
    /// another device.
    pub async fn delete_device(self) -> Result<(), (Self, ClientError)> {
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
            .delete_device(account_id, device_id, &password, device_id)
            .await;

        let Ok(()) = delete_result else {
            return Err((self, ClientError::Api(delete_result.unwrap_err())));
        };

        Ok(())
    }

    /// Unlink another device from the client's account.
    /// This can only be done from the primary device.
    pub async fn unlink_device(self, device_id: DeviceId) -> Result<(), ClientError> {
        self.api_client
            .delete_device(
                self.account_id().await?,
                self.device_id().await?,
                &self.store.account_store.get_password().await?,
                device_id,
            )
            .await?;
        Ok(())
    }

    /// Get the [AccountId] of a user by username.
    pub async fn get_account_id_for(&self, username: &str) -> Result<AccountId, ClientError> {
        let account_id = self
            .api_client
            .get_user_account_id(
                self.account_id().await?,
                self.device_id().await?,
                self.store.account_store.get_password().await?.as_str(),
                username,
            )
            .await?;

        Ok(account_id)
    }

    pub async fn remove_device_for(
        &mut self,
        account_id: AccountId,
        device_id: DeviceId,
    ) -> Result<(), ClientError> {
        self.store
            .contact_store
            .remove_device(account_id, device_id)
            .await
    }

    pub async fn disconnect(&mut self) -> Result<(), ClientError> {
        self.protocol_client
            .disconnect()
            .await
            .map_err(ClientError::from)
    }

    pub async fn connect(&mut self) -> Result<(), ClientError> {
        self.envelope_queue = self.protocol_client.connect().await?;
        Ok(())
    }

    pub async fn is_connected(&self) -> bool {
        self.protocol_client.is_connected().await
    }

    /// Send any message to recipient
    /// Should also send to users other devices
    pub async fn send_message(
        &mut self,
        recipient: AccountId,
        msg: impl Into<Vec<u8>>,
    ) -> Result<MessageStatus, ClientError> {
        if !self.store.contact_store.contains_contact(recipient).await? {
            return Err(ClientError::NoContact);
        }
        let envelope = encrypt(msg, vec![recipient], &mut self.store).await?;
        self.protocol_client
            .send_message(envelope)
            .await
            .map_err(ClientError::from)
    }

    /// Returns a broadcast receiver for incoming messages that have been decrypted
    pub fn subscribe(&self) -> Receiver<DecryptedEnvelope> {
        self.store.message_store.subscribe()
    }

    async fn _process_messages(&mut self, block: bool) -> Result<(), ClientError> {
        if !block && self.envelope_queue.is_empty() {
            return Ok(());
        }
        while let Some(envelope) = self.envelope_queue.recv().await {
            // TODO: How should we handle failure to decrypt and/or store message?
            let envelope = match decrypt(envelope, &mut self.store).await {
                Ok(denvelope) => denvelope,
                Err(e) => {
                    error!("Failed to decrypt message: {e}");
                    break;
                }
            };

            let _ = self
                .store
                .message_store
                .store_message(envelope)
                .await
                .inspect_err(|e| error!("Failed to store message {e}"));
            if self.envelope_queue.is_empty() {
                break;
            }
        }
        Ok(())
    }

    pub async fn process_messages_blocking(&mut self) -> Result<(), ClientError> {
        self._process_messages(true).await
    }

    pub async fn process_messages(&mut self) -> Result<(), ClientError> {
        self._process_messages(false).await
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
                self.device_id().await?,
                self.store.account_store.get_password().await?.as_str(),
                pre_key_bundle,
            )
            .await?;

        Ok(())
    }

    /// Fetch key bundles for account_id
    pub async fn fetch_prekeys(
        &mut self,
        account_id: AccountId,
        devices: Option<Vec<DeviceId>>,
    ) -> Result<(), ClientError> {
        let prekey_bundles = self
            .api_client
            .get_pre_key_bundles(
                self.account_id().await?,
                self.device_id().await?,
                self.store.account_store.get_password().await?.as_str(),
                account_id,
                devices,
            )
            .await?;

        let time = SystemTime::now();

        for bundle in prekey_bundles.bundles {
            let device_id = bundle.device_id;
            self.store
                .contact_store
                .add_device(account_id, device_id.into())
                .await?;
            let libsignal_bundle = into_libsignal_bundle(bundle, prekey_bundles.identity_key)
                .map_err(|_| ClientError::FailedToConvertPreKeyBundle)?;
            process_prekey_bundle(
                &ProtocolAddress::new(account_id.to_string(), device_id.into()),
                &mut self.store.session_store,
                &mut self.store.identity_key_store,
                &libsignal_bundle,
                time,
                &mut OsRng,
            )
            .await
            .map_err(|_| ClientError::FailedToProcessPrekeyBundle)?;
        }

        Ok(())
    }

    /// Create a provision token to be used on another client to activate
    pub async fn create_provision(&mut self) -> Result<LinkDeviceToken, ClientError> {
        Ok(self
            .api_client
            .provision_device(
                self.account_id().await?,
                self.device_id().await?,
                &self.store.account_store.get_password().await?,
            )
            .await?)
    }
}
