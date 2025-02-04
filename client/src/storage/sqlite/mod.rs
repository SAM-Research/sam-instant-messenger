use account::SqliteAccountStore;
use async_trait::async_trait;
use contact::SqliteContactStore;
use identity::SqliteIdentityKeyStore;
use kyber::SqliteKyberPreKeyStore;
use libsignal_protocol::IdentityKeyPair;
use pre_key::SqlitePreKeyStore;
use sender_key::SqliteSessionStore;
use session::SqliteSenderKeyStore;
use signed_pre_key::SqliteSignedPreKeyStore;
use sqlx::{Pool, Sqlite};

use crate::ClientError;

use super::{Store, StoreConfig, StoreType};

pub mod account;
pub mod contact;
pub mod identity;
pub mod kyber;
pub mod pre_key;
pub mod sender_key;
pub mod session;
pub mod signed_pre_key;

#[derive(Debug)]
pub struct SqliteStoreType;

impl StoreType for SqliteStoreType {
    type ContactStore = SqliteContactStore;

    type AccountStore = SqliteAccountStore;

    type IdentityKeyStore = SqliteIdentityKeyStore;

    type PreKeyStore = SqlitePreKeyStore;

    type SignedPreKeyStore = SqliteSignedPreKeyStore;

    type KyberPreKeyStore = SqliteKyberPreKeyStore;

    type SessionStore = SqliteSessionStore;

    type SenderKeyStore = SqliteSenderKeyStore;
}

pub type SqliteStore = Store<SqliteStoreType>;
#[derive(Debug)]
pub struct SqliteStoreConfig {
    database: Pool<Sqlite>,
}

#[async_trait(?Send)]
impl StoreConfig for SqliteStoreConfig {
    type StoreType = SqliteStoreType;
    async fn create_store<ID: Into<u32>>(
        self,
        key_pair: IdentityKeyPair,
        registration_id: ID,
    ) -> Result<SqliteStore, ClientError> {
        Ok(SqliteStore::builder()
            .contact_store(SqliteContactStore::new(self.database.clone()))
            .account_store(SqliteAccountStore::new(self.database.clone()))
            .identity_key_store(SqliteIdentityKeyStore::new(
                self.database.clone(),
                key_pair,
                registration_id.into(),
            ))
            .pre_key_store(SqlitePreKeyStore::new(self.database.clone()))
            .signed_pre_key_store(SqliteSignedPreKeyStore::new(self.database.clone()))
            .kyber_pre_key_store(SqliteKyberPreKeyStore::new(self.database.clone()))
            .sender_key_store(SqliteSenderKeyStore::new(self.database.clone()))
            .session_store(SqliteSessionStore::new(self.database.clone()))
            .build())
    }

    async fn load_store(self) -> Result<SqliteStore, ClientError> {
        todo!()
    }
}
