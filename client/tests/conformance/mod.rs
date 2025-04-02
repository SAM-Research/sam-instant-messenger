use libsignal_core::ProtocolAddress;
use libsignal_protocol::IdentityKeyPair;
use rand::rngs::OsRng;
use sam_client::storage::inmem::{
    InMemorySamStore, InMemorySamStoreConfig, InMemorySignalStore, InMemorySignalStoreConfig,
};
use sam_client::storage::sqlite::{
    SqliteSamStore, SqliteSamStoreConfig, SqliteSignalStore, SqliteSignalStoreConfig,
};
use sam_client::storage::{SamStoreConfig, SignalStoreConfig};

mod account;
mod contact;
mod identity;
mod kyber;
mod pre_key;
mod session;
mod signed_pre_key;

async fn sam_sqlite() -> SqliteSamStore {
    SqliteSamStoreConfig::in_memory()
        .await
        .create_store()
        .await
        .unwrap()
}

async fn sam_in_mem() -> InMemorySamStore {
    InMemorySamStoreConfig::default()
        .create_store()
        .await
        .unwrap()
}

async fn signal_sqlite() -> SqliteSignalStore {
    let key_pair = IdentityKeyPair::generate(&mut OsRng);
    let registration_id = 0u32;
    SqliteSignalStoreConfig::in_memory()
        .await
        .create_store(key_pair, registration_id)
        .await
        .unwrap()
}

async fn signal_in_mem() -> InMemorySignalStore {
    let key_pair = IdentityKeyPair::generate(&mut OsRng);
    let registration_id = 0u32;
    InMemorySignalStoreConfig::default()
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
