use super::{Store, StoreType};
use account::InMemoryAccountStore;
use contact::InMemoryContactStore;
use libsignal_protocol::{
    InMemIdentityKeyStore, InMemKyberPreKeyStore, InMemPreKeyStore, InMemSenderKeyStore,
    InMemSessionStore, InMemSignedPreKeyStore,
};

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

#[derive(Debug)]
pub struct InMemoryStoreConfig {}
