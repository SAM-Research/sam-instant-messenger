mod account;
mod identity;
mod kyber;
mod pre_key;
mod sender_key;
mod session;
mod signed_pre_key;

use super::{
    inmem::{InMemoryStore, InMemoryStoreConfig},
    sqlite::{SqliteStore, SqliteStoreConfig},
    StoreConfig,
};
use libsignal_core::ProtocolAddress;
use libsignal_protocol::IdentityKeyPair;
use rand::rngs::OsRng;

async fn sqlite() -> SqliteStore {
    let key_pair = IdentityKeyPair::generate(&mut OsRng);
    let registration_id = 0u32;
    SqliteStoreConfig::in_memory()
        .await
        .create_store(key_pair, registration_id)
        .await
        .unwrap()
}

async fn in_mem() -> InMemoryStore {
    let key_pair = IdentityKeyPair::generate(&mut OsRng);
    let registration_id = 0u32;
    InMemoryStoreConfig::default()
        .create_store(key_pair, registration_id)
        .await
        .unwrap()
}
pub fn alice_address() -> ProtocolAddress {
    ProtocolAddress::new("alice".to_owned(), 0.into())
}
pub fn bob_address() -> ProtocolAddress {
    ProtocolAddress::new("bob".to_owned(), 0.into())
}
pub fn key_pair() -> IdentityKeyPair {
    IdentityKeyPair::generate(&mut OsRng)
}
