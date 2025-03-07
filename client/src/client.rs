use bon::bon;
use sam_common::{api::LinkDeviceToken, AccountId, DeviceId};
use tokio::sync::mpsc::Receiver;

use crate::{
    encryption::envelope::DecryptedEnvelope,
    net::{
        api_trait::ApiClientConfig,
        protocol::traits::{ProtocolConfig, SamProtocolClient},
        ApiClient,
    },
    storage::{AccountStore, Store, StoreConfig, StoreType},
    ClientError,
};

pub struct Client<T: StoreType, U: ApiClient, V: SamProtocolClient> {
    store: Store<T>,
    _api_client: U,
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
        _store_config: impl StoreConfig<StoreType = T>,
        _protocol_config: impl ProtocolConfig,
        _api_client_config: impl ApiClientConfig,
        _username: &str,
        _password: &str,
        _device_name: &str,
    ) -> Result<Self, ClientError> {
        todo!()
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
