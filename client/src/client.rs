use bon::bon;
use libsignal_protocol::{IdentityKeyPair, IdentityKeyStore};
use rand::rngs::OsRng;
use rand::{CryptoRng, Rng};
use sam_common::{
    address::RegistrationId, api::LinkDeviceToken, sam_message::ServerEnvelope, AccountId, DeviceId,
};

use tokio::sync::broadcast::Receiver;
use tokio::sync::mpsc::Receiver as MpscReceiver;

use crate::logic::{handle_message_response, prepare_message, provision_device};
use crate::net::protocol::ProtocolClient;
use crate::net::HttpClient;
use crate::storage::{InMemoryStoreType, SqliteStoreType};
use crate::{
    encryption::envelope::DecryptedEnvelope,
    logic::{process_messages, publish_prekeys, register_account},
    net::{
        api_trait::ApiClientConfig,
        protocol::traits::{ProtocolConfig, SamProtocolClient},
        ApiClient,
    },
    storage::{traits::message::MessageStore, AccountStore, Store, StoreConfig, StoreType},
    ClientError,
};
pub trait ClientType {
    type Store: StoreType;
    type ApiClient: ApiClient;
    type ProtocolClient: SamProtocolClient;
    type Rng: Rng + CryptoRng + Default;
}

pub struct DefaultClientType<T: StoreType, U: ApiClient, V: SamProtocolClient> {
    _store: std::marker::PhantomData<T>,
    _api: std::marker::PhantomData<U>,
    _protocol: std::marker::PhantomData<V>,
}

impl<T: StoreType, U: ApiClient, V: SamProtocolClient> ClientType for DefaultClientType<T, U, V> {
    type Store = T;

    type ApiClient = U;

    type ProtocolClient = V;

    type Rng = OsRng;
}

pub type InMemoryClientType = DefaultClientType<InMemoryStoreType, HttpClient, ProtocolClient>;
pub type SqliteClientType = DefaultClientType<SqliteStoreType, HttpClient, ProtocolClient>;

pub struct Client<T: ClientType> {
    store: Store<T::Store>,
    api_client: T::ApiClient,
    protocol_client: T::ProtocolClient,
    envelope_queue: MpscReceiver<ServerEnvelope>,
    rng: T::Rng,
}

#[bon]
impl<T: ClientType> Client<T> {
    /// Creates a new client for the account described in the token
    #[builder]
    pub async fn from_provisioning(
        store_config: impl StoreConfig<StoreType = T::Store>,
        protocol_config: impl ProtocolConfig<ProtocolClient = T::ProtocolClient>,
        api_client_config: impl ApiClientConfig<ApiClient = T::ApiClient>,
        device_name: &str,
        id_key_pair: IdentityKeyPair,
        token: LinkDeviceToken,
        #[builder(default = 100)] upload_prekey_count: usize,
        #[builder(default = 16)] password_length: usize,
        #[builder(default = <T::Rng as Default>::default())] mut rng: T::Rng,
    ) -> Result<Self, ClientError> {
        let api_client = api_client_config.create().await?;
        let registration_id = RegistrationId::generate(&mut rng);

        let mut store = store_config
            .create_store(id_key_pair, registration_id)
            .await?;

        provision_device(
            &api_client,
            &mut store,
            device_name,
            token,
            upload_prekey_count,
            password_length,
            &mut rng,
        )
        .await?;

        let mut protocol_client = protocol_config.create(
            store.account_store.get_account_id().await?,
            store.account_store.get_device_id().await?,
            store.account_store.get_password().await?,
        )?;

        let queue = protocol_client.connect().await?;

        Ok(Self {
            store,
            api_client,
            protocol_client,
            envelope_queue: queue,
            rng,
        })
    }

    /// Register a new account.
    #[builder]
    pub async fn from_registration(
        store_config: impl StoreConfig<StoreType = T::Store>,
        protocol_config: impl ProtocolConfig<ProtocolClient = T::ProtocolClient>,
        api_client_config: impl ApiClientConfig<ApiClient = T::ApiClient>,
        username: &str,
        device_name: &str,

        #[builder(default = 100)] upload_prekey_count: usize,
        #[builder(default = 16)] password_length: usize,
        #[builder(default = <T::Rng as Default>::default())] mut rng: T::Rng,
    ) -> Result<Self, ClientError> {
        let registration_id = RegistrationId::generate(&mut rng);
        let id_key_pair = IdentityKeyPair::generate(&mut rng);
        let mut store = store_config
            .create_store(id_key_pair, registration_id)
            .await?;
        let api_client = api_client_config.create().await?;

        register_account(
            &api_client,
            &mut store,
            username,
            device_name,
            password_length,
            upload_prekey_count,
            &mut rng,
        )
        .await?;

        let mut protocol_client = protocol_config.create(
            store.account_store.get_account_id().await?,
            store.account_store.get_device_id().await?,
            store.account_store.get_password().await?,
        )?;
        let queue = protocol_client.connect().await?;

        Ok(Self {
            store,
            protocol_client,
            api_client,
            envelope_queue: queue,
            rng,
        })
    }

    /// Instantiate a client from a valid store.
    #[builder]
    pub async fn from_store(
        store: Store<T::Store>,
        protocol_config: impl ProtocolConfig<ProtocolClient = T::ProtocolClient>,
        api_client_config: impl ApiClientConfig<ApiClient = T::ApiClient>,
        #[builder(default = <T::Rng as Default>::default())] rng: T::Rng,
    ) -> Result<Self, ClientError> {
        let account_id = store.account_store.get_account_id().await?;
        let device_id = store.account_store.get_device_id().await?;
        let password = store.account_store.get_password().await?;
        let mut protocol_client = protocol_config.create(account_id, device_id, password)?;
        let queue = protocol_client.connect().await?;
        Ok(Self {
            store,
            protocol_client,
            api_client: api_client_config.create().await?,
            envelope_queue: queue,
            rng,
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

    /// Delete Account and consumes the client.
    /// If account deletion fails, the client is returned along with the error.
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

    /// Delete this device and consumes the client.
    /// This cannot be done for the primary device.
    ///
    /// See `unlink_device` if you want to delete another device.
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

    /// Disconnect from the server.
    pub async fn disconnect(&mut self) -> Result<(), ClientError> {
        self.protocol_client
            .disconnect()
            .await
            .map_err(ClientError::from)
    }

    /// Connect to the server to recieve messages.
    pub async fn connect(&mut self) -> Result<(), ClientError> {
        self.envelope_queue = self.protocol_client.connect().await?;
        Ok(())
    }

    /// Returns whether or not the client is connected to the server.
    pub async fn is_connected(&self) -> bool {
        self.protocol_client.is_connected().await
    }

    /// Send any message to recipient. Also sends syncs the message with your other devices.
    pub async fn send_message(
        &mut self,
        recipient: AccountId,
        msg: impl Into<Vec<u8>>,
    ) -> Result<(), ClientError> {
        let client_envelope = prepare_message(
            &mut self.store,
            &self.api_client,
            recipient,
            msg,
            &mut self.rng,
        )
        .await?;
        let status = self.protocol_client.send_message(client_envelope).await?;
        handle_message_response(&mut self.store, &self.api_client, &mut self.rng, status).await?;
        Ok(())
    }

    /// Returns a broadcast receiver for incoming messages that have been decrypted.
    pub fn subscribe(&self) -> Receiver<DecryptedEnvelope> {
        self.store.message_store.subscribe()
    }

    /// Recieve and decrypt messages. Block until at least one message is received.
    pub async fn process_messages_blocking(&mut self) -> Result<(), ClientError> {
        process_messages(&mut self.store, &mut self.envelope_queue, true).await
    }

    /// Recieve and decrypt messages.
    pub async fn process_messages(&mut self) -> Result<(), ClientError> {
        process_messages(&mut self.store, &mut self.envelope_queue, false).await
    }

    /// Publish new prekeys.
    #[builder]
    pub async fn publish_prekeys(
        &mut self,
        #[builder(default)] onetime_prekeys: usize,
        #[builder(default = false)] new_signed_prekey: bool,
        #[builder(default = false)] new_last_resort: bool,
    ) -> Result<(), ClientError> {
        publish_prekeys(
            &mut self.store,
            &self.api_client,
            onetime_prekeys,
            new_signed_prekey,
            new_last_resort,
            &mut self.rng,
        )
        .await
    }

    /// Create a provisioning token for linking a new device to your account.
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
