use crate::ClientError;
use async_trait::async_trait;
use bon::Builder;
use libsignal_protocol::{
    IdentityKeyPair, IdentityKeyStore, KyberPreKeyId, KyberPreKeyStore, PreKeyId, PreKeyStore,
    SenderKeyStore, SessionStore, SignedPreKeyId, SignedPreKeyStore,
};
use std::fmt::Debug;
pub mod error;

pub use traits::{account::AccountStore, contact::ContactStore};

pub mod inmem;
pub mod sqlite;
pub mod traits;

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

#[async_trait(?Send)]
pub trait ProvidesKeyId<T> {
    async fn next_key_id(&self) -> Result<T, ClientError>;
}

pub trait StoreType {
    type ContactStore: ContactStore;
    type AccountStore: AccountStore;
    type IdentityKeyStore: IdentityKeyStore;
    type PreKeyStore: PreKeyStore + ProvidesKeyId<PreKeyId>;
    type SignedPreKeyStore: SignedPreKeyStore + ProvidesKeyId<SignedPreKeyId>;
    type KyberPreKeyStore: KyberPreKeyStore + ProvidesKeyId<KyberPreKeyId>;
    type SessionStore: SessionStore;
    type SenderKeyStore: SenderKeyStore;
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
