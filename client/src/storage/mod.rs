use crate::ClientError;
use async_trait::async_trait;
use bon::Builder;
use libsignal_protocol::{
    Aci, IdentityKeyPair, IdentityKeyStore, KyberPreKeyStore, PreKeyStore, SenderKeyStore,
    SessionStore, SignedPreKeyStore,
};
use std::fmt::Debug;
pub mod sqlite;

#[async_trait(?Send)]
pub trait StoreConfig {
    type StoreType: StoreType;

    async fn create_store<ID: Into<u32>>(
        self,
        key_pair: IdentityKeyPair,
        registration_id: ID,
    ) -> Result<Store<Self::StoreType>, ClientError>;

    async fn load_store(self) -> Result<Store<Self::StoreType>, ClientError>;
}

pub trait ContactStore {}

#[async_trait(?Send)]
pub trait AccountStore {
    async fn set_aci(&self, aci: Aci) -> Result<(), ClientError>;
    async fn get_aci(&self) -> Result<Aci, ClientError>;
    async fn set_password(&self, password: String) -> Result<(), ClientError>;
    async fn get_password(&self) -> Result<String, ClientError>;
    async fn set_username(&self, username: String) -> Result<(), ClientError>;
    async fn get_username(&self) -> Result<String, ClientError>;
}

#[async_trait(?Send)]
pub trait ProvidesKeyId {
    type KeyIdType;

    async fn next_key_id(&self) -> Result<Self::KeyIdType, ClientError>;
}

pub trait StoreType {
    type ContactStore: ContactStore + Debug;
    type AccountStore: AccountStore + Debug;
    type IdentityKeyStore: IdentityKeyStore + Debug;
    type PreKeyStore: PreKeyStore + ProvidesKeyId + Debug;
    type SignedPreKeyStore: SignedPreKeyStore + Debug;
    type KyberPreKeyStore: KyberPreKeyStore + ProvidesKeyId + Debug;
    type SessionStore: SessionStore + Debug;
    type SenderKeyStore: SenderKeyStore + Debug;
}

#[derive(Debug, Builder)]
pub struct Store<T: StoreType> {
    pub contact_store: T::ContactStore,
    pub account_store: T::AccountStore,
    pub identity_key_store: T::IdentityKeyStore,
    pub pre_key_store: T::PreKeyStore,
    pub signed_pre_key_store: T::SignedPreKeyStore,
    pub kyber_pre_key_store: T::KyberPreKeyStore,
    pub session_store: T::SessionStore,
    pub sender_key_store: T::SenderKeyStore,
}
