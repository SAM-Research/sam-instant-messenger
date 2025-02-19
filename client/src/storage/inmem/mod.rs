use crate::ClientError;

use super::{Store, StoreConfig, StoreType};
use account::InMemoryAccountStore;
use async_trait::async_trait;
use contact::InMemoryContactStore;
use libsignal_protocol::{
    IdentityKeyPair, InMemIdentityKeyStore, InMemKyberPreKeyStore, InMemPreKeyStore,
    InMemSenderKeyStore, InMemSessionStore, InMemSignedPreKeyStore,
};
use rand::rngs::OsRng;
use sam_common::address::RegistrationId;

pub mod account;
pub mod contact;
pub mod kyber;
pub mod pre_key;
pub mod signed_pre_key;

#[derive(Debug)]
pub struct InMemoryStoreType;

impl StoreType for InMemoryStoreType {
    type ContactStore = InMemoryContactStore;

    type AccountStore = InMemoryAccountStore;

    type IdentityKeyStore = InMemIdentityKeyStore;

    type PreKeyStore = InMemPreKeyStore;

    type SignedPreKeyStore = InMemSignedPreKeyStore;

    type KyberPreKeyStore = InMemKyberPreKeyStore;

    type SessionStore = InMemSessionStore;

    type SenderKeyStore = InMemSenderKeyStore;
}

pub type InMemoryStore = Store<InMemoryStoreType>;

#[derive(Debug, Default)]
pub struct InMemoryStoreConfig {}

#[async_trait(?Send)]
impl StoreConfig for InMemoryStoreConfig {
    type StoreType = InMemoryStoreType;
    async fn create_store<ID: Into<u32>>(
        self,
        key_pair: IdentityKeyPair,
        registration_id: ID,
    ) -> Result<InMemoryStore, ClientError> {
        Ok(InMemoryStore::builder()
            .identity_key_store(InMemIdentityKeyStore::new(key_pair, registration_id.into()))
            .pre_key_store(InMemPreKeyStore::default())
            .signed_pre_key_store(InMemSignedPreKeyStore::default())
            .kyber_pre_key_store(InMemKyberPreKeyStore::default())
            .sender_key_store(InMemSenderKeyStore::default())
            .session_store(InMemSessionStore::default())
            .account_store(InMemoryAccountStore::default())
            .contact_store(InMemoryContactStore::default())
            .build())
    }

    async fn load_store(self) -> Result<InMemoryStore, ClientError> {
        let mut csprng = OsRng;
        self.create_store(
            IdentityKeyPair::generate(&mut csprng),
            RegistrationId::generate(&mut csprng),
        )
        .await
    }
}
