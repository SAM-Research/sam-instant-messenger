use crate::ClientError;
use async_trait::async_trait;
use bon::Builder;
use libsignal_protocol::{
    IdentityKeyPair, IdentityKeyStore, KyberPreKeyId, KyberPreKeyRecord, KyberPreKeyStore,
    PreKeyId, PreKeyRecord, PreKeyStore, SenderKeyStore, SessionStore, SignedPreKeyId,
    SignedPreKeyRecord, SignedPreKeyStore,
};
use std::fmt::Debug;

use crate::storage::key_generation::{
    KyberKeyGeneration, PreKeyGeneration, SignedPreKeyGeneration,
};
pub use traits::{account::AccountStore, contact::ContactStore};

pub mod inmem;
pub mod key_generation;
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
    type PreKeyStore: PreKeyStore + ProvidesKeyId<PreKeyId> + PreKeyGeneration<PreKeyRecord>;
    type SignedPreKeyStore: SignedPreKeyStore
        + ProvidesKeyId<SignedPreKeyId>
        + SignedPreKeyGeneration<SignedPreKeyRecord>;
    type KyberPreKeyStore: KyberPreKeyStore
        + ProvidesKeyId<KyberPreKeyId>
        + KyberKeyGeneration<KyberPreKeyRecord>;
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
