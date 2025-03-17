use super::{Store, StoreConfig, StoreType};
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

    type MessageStore = InMemoryMessageStore;
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
            .message_store(InMemoryMessageStore::new(10))
            .build())
    }
}
