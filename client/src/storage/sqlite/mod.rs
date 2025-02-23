use account::SqliteAccountStore;
use async_trait::async_trait;
use contact::SqliteContactStore;
use identity::SqliteIdentityKeyStore;
use kyber::SqliteKyberPreKeyStore;
use libsignal_protocol::IdentityKeyPair;
use pre_key::SqlitePreKeyStore;
use sender_key::SqliteSenderKeyStore;
use session::SqliteSessionStore;
use signed_pre_key::SqliteSignedPreKeyStore;
use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};

use crate::ClientError;

use super::{
    store_builder::{
        SetAccountStore, SetContactStore, SetKyberPreKeyStore, SetPreKeyStore, SetSenderKeyStore,
        SetSessionStore, SetSignedPreKeyStore,
    },
    Store, StoreBuilder, StoreConfig, StoreType,
};

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

pub async fn connect_to_in_memory() -> Pool<Sqlite> {
    let db_url = "sqlite::memory:".to_owned();
    let pool = SqlitePoolOptions::new()
        .connect(&db_url)
        .await
        .expect("Could not connect to database");
    sqlx::migrate!("database/migrations")
        .run(&pool)
        .await
        .expect("should be able to run migrations");

    pool
}

impl SqliteStoreConfig {
    pub fn new(database: Pool<Sqlite>) -> Self {
        Self { database }
    }
    pub async fn in_memory() -> Self {
        let database = connect_to_in_memory().await;
        Self { database }
    }
}

type BuilderProperties = SetSessionStore<
    SetSenderKeyStore<
        SetKyberPreKeyStore<SetSignedPreKeyStore<SetPreKeyStore<SetAccountStore<SetContactStore>>>>,
    >,
>;
type PreparedStoreBuilder = StoreBuilder<SqliteStoreType, BuilderProperties>;
fn pre_build_store(database: Pool<Sqlite>) -> PreparedStoreBuilder {
    SqliteStore::builder()
        .contact_store(SqliteContactStore::new(database.clone()))
        .account_store(SqliteAccountStore::new(database.clone()))
        .pre_key_store(SqlitePreKeyStore::new(database.clone()))
        .signed_pre_key_store(SqliteSignedPreKeyStore::new(database.clone()))
        .kyber_pre_key_store(SqliteKyberPreKeyStore::new(database.clone()))
        .sender_key_store(SqliteSenderKeyStore::new(database.clone()))
        .session_store(SqliteSessionStore::new(database.clone()))
}

#[async_trait(?Send)]
impl StoreConfig for SqliteStoreConfig {
    type StoreType = SqliteStoreType;
    async fn create_store<ID: Into<u32>>(
        self,
        key_pair: IdentityKeyPair,
        registration_id: ID,
    ) -> Result<SqliteStore, ClientError> {
        Ok(pre_build_store(self.database.clone())
            .identity_key_store(
                SqliteIdentityKeyStore::create(
                    self.database.clone(),
                    key_pair,
                    registration_id.into(),
                )
                .await?,
            )
            .build())
    }

    async fn load_store(self) -> Result<SqliteStore, ClientError> {
        Ok(pre_build_store(self.database.clone())
            .identity_key_store(SqliteIdentityKeyStore::load(self.database.clone()).await?)
            .build())
    }
}
