use super::{
    SamStore, SamStoreConfig, SamStoreType, SignalStore, SignalStoreConfig, SignalStoreType,
};
use crate::ClientError;
pub use account::InMemoryAccountStore;
use async_trait::async_trait;
pub use contact::InMemoryContactStore;
use libsignal_protocol::{
    IdentityKeyPair, InMemIdentityKeyStore, InMemKyberPreKeyStore, InMemPreKeyStore,
    InMemSenderKeyStore, InMemSessionStore, InMemSignedPreKeyStore,
};
pub use message::InMemoryMessageStore;

pub mod account;
pub mod contact;
pub mod kyber;
pub mod message;
pub mod pre_key;
pub mod signed_pre_key;

#[derive(Debug)]
pub struct InMemorySignalStoreType;

impl SignalStoreType for InMemorySignalStoreType {
    type IdentityKeyStore = InMemIdentityKeyStore;

    type PreKeyStore = InMemPreKeyStore;

    type SignedPreKeyStore = InMemSignedPreKeyStore;

    type KyberPreKeyStore = InMemKyberPreKeyStore;

    type SessionStore = InMemSessionStore;

    type SenderKeyStore = InMemSenderKeyStore;
}

pub struct InMemorySamStoreType;

impl SamStoreType for InMemorySamStoreType {
    type AccountStore = InMemoryAccountStore;
    type ContactStore = InMemoryContactStore;
    type MessageStore = InMemoryMessageStore;
}

pub type InMemorySignalStore = SignalStore<InMemorySignalStoreType>;
pub type InMemorySamStore = SamStore<InMemorySamStoreType>;

#[derive(Debug, Default)]
pub struct InMemorySignalStoreConfig {}

#[derive(Debug, Default)]
pub struct InMemorySamStoreConfig {}

#[async_trait(?Send)]
impl SignalStoreConfig for InMemorySignalStoreConfig {
    type StoreType = InMemorySignalStoreType;
    async fn create_store<ID: Into<u32>>(
        self,
        key_pair: IdentityKeyPair,
        registration_id: ID,
    ) -> Result<InMemorySignalStore, ClientError> {
        Ok(InMemorySignalStore::builder()
            .identity_key_store(InMemIdentityKeyStore::new(key_pair, registration_id.into()))
            .pre_key_store(InMemPreKeyStore::default())
            .signed_pre_key_store(InMemSignedPreKeyStore::default())
            .kyber_pre_key_store(InMemKyberPreKeyStore::default())
            .sender_key_store(InMemSenderKeyStore::default())
            .session_store(InMemSessionStore::default())
            .build())
    }
}

#[async_trait]
impl SamStoreConfig for InMemorySamStoreConfig {
    type StoreType = InMemorySamStoreType;

    async fn create_store(self) -> Result<InMemorySamStore, ClientError> {
        Ok(InMemorySamStore::builder()
            .account_store(InMemoryAccountStore::default())
            .contact_store(InMemoryContactStore::default())
            .message_store(InMemoryMessageStore::new(10))
            .build())
    }
}
