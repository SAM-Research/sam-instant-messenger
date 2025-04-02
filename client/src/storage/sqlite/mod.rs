pub use account::SqliteAccountStore;
use async_trait::async_trait;
pub use contact::SqliteContactStore;
pub use identity::SqliteIdentityKeyStore;
pub use kyber::SqliteKyberPreKeyStore;
use libsignal_protocol::IdentityKeyPair;
pub use message::SqliteMessageStore;
pub use pre_key::SqlitePreKeyStore;
pub use sender_key::SqliteSenderKeyStore;
pub use session::SqliteSessionStore;
pub use signed_pre_key::SqliteSignedPreKeyStore;
use sqlx::sqlite::SqlitePoolOptions;

use super::{
    SamStore, SamStoreConfig, SamStoreType, SignalStore, SignalStoreConfig, SignalStoreType,
};
use crate::ClientError;

pub mod account;
pub mod contact;
pub mod identity;
pub mod kyber;
pub mod message;
pub mod pre_key;
pub mod sender_key;
pub mod session;
pub mod signed_pre_key;

#[derive(Debug)]
pub struct SqliteSignalStoreType;

impl SignalStoreType for SqliteSignalStoreType {
    type IdentityKeyStore = SqliteIdentityKeyStore;

    type PreKeyStore = SqlitePreKeyStore;

    type SignedPreKeyStore = SqliteSignedPreKeyStore;

    type KyberPreKeyStore = SqliteKyberPreKeyStore;

    type SessionStore = SqliteSessionStore;

    type SenderKeyStore = SqliteSenderKeyStore;
}

pub struct SqliteSamStoreType;

impl SamStoreType for SqliteSamStoreType {
    type AccountStore = SqliteAccountStore;

    type ContactStore = SqliteContactStore;

    type MessageStore = SqliteMessageStore;
}

pub type SqliteSignalStore = SignalStore<SqliteSignalStoreType>;
pub type SqliteSamStore = SamStore<SqliteSamStoreType>;
#[derive(Debug)]
pub struct SqliteSignalStoreConfig {
    connection_string: String,
}

pub struct SqliteSamStoreConfig {
    connection_string: String,
}

impl SqliteSignalStoreConfig {
    pub fn new(connection_string: String) -> Self {
        Self { connection_string }
    }

    pub async fn in_memory() -> Self {
        Self {
            connection_string: "sqlite::memory:".to_owned(),
        }
    }

    /// Load an existing database.
    pub async fn load(self) -> Result<SqliteSignalStore, ClientError> {
        let database = SqlitePoolOptions::new()
            .connect(&self.connection_string)
            .await
            .map_err(|err| {
                ClientError::Database(format!(
                    "Could not connect to the database at '{}': {}",
                    self.connection_string, err
                ))
            })?;

        Ok(SqliteSignalStore::builder()
            .pre_key_store(SqlitePreKeyStore::new(database.clone()))
            .signed_pre_key_store(SqliteSignedPreKeyStore::new(database.clone()))
            .kyber_pre_key_store(SqliteKyberPreKeyStore::new(database.clone()))
            .sender_key_store(SqliteSenderKeyStore::new(database.clone()))
            .session_store(SqliteSessionStore::new(database.clone()))
            .identity_key_store(SqliteIdentityKeyStore::load(database.clone()).await?)
            .build())
    }
}

impl SqliteSamStoreConfig {
    pub fn new(connection_string: String) -> Self {
        Self { connection_string }
    }

    pub async fn in_memory() -> Self {
        Self {
            connection_string: "sqlite::memory:".to_owned(),
        }
    }

    /// Load an existing database.
    pub async fn load(self) -> Result<SqliteSamStore, ClientError> {
        let database = SqlitePoolOptions::new()
            .connect(&self.connection_string)
            .await
            .map_err(|err| {
                ClientError::Database(format!(
                    "Could not connect to the database at '{}': {}",
                    self.connection_string, err
                ))
            })?;

        Ok(SqliteSamStore::builder()
            .account_store(SqliteAccountStore::new(database.clone()))
            .contact_store(SqliteContactStore::new(database.clone()))
            .message_store(SqliteMessageStore::new(database.clone(), 10))
            .build())
    }
}

#[async_trait(?Send)]
impl SignalStoreConfig for SqliteSignalStoreConfig {
    type StoreType = SqliteSignalStoreType;

    /// Create a new database and run migrations.
    async fn create_store<ID: Into<u32>>(
        self,
        key_pair: IdentityKeyPair,
        registration_id: ID,
    ) -> Result<SqliteSignalStore, ClientError> {
        let database = SqlitePoolOptions::new()
            .connect(&self.connection_string)
            .await
            .map_err(|err| {
                ClientError::Database(format!(
                    "Could not connect to the database at '{}': {}",
                    self.connection_string, err
                ))
            })?;
        sqlx::migrate!("database/migrations")
            .run(&database)
            .await
            .map_err(|err| {
                ClientError::Database(format!(
                    "Could not run migrations on database at '{}': {}",
                    self.connection_string, err
                ))
            })?;

        Ok(SqliteSignalStore::builder()
            .pre_key_store(SqlitePreKeyStore::new(database.clone()))
            .signed_pre_key_store(SqliteSignedPreKeyStore::new(database.clone()))
            .kyber_pre_key_store(SqliteKyberPreKeyStore::new(database.clone()))
            .sender_key_store(SqliteSenderKeyStore::new(database.clone()))
            .session_store(SqliteSessionStore::new(database.clone()))
            .identity_key_store(
                SqliteIdentityKeyStore::new(database.clone(), key_pair, registration_id.into())
                    .await?,
            )
            .build())
    }
}

#[async_trait]
impl SamStoreConfig for SqliteSamStoreConfig {
    type StoreType = SqliteSamStoreType;

    async fn create_store(self) -> Result<SqliteSamStore, ClientError> {
        let database = SqlitePoolOptions::new()
            .connect(&self.connection_string)
            .await
            .map_err(|err| {
                ClientError::Database(format!(
                    "Could not connect to the database at '{}': {}",
                    self.connection_string, err
                ))
            })?;
        sqlx::migrate!("database/migrations")
            .run(&database)
            .await
            .map_err(|err| {
                ClientError::Database(format!(
                    "Could not run migrations on database at '{}': {}",
                    self.connection_string, err
                ))
            })?;

        Ok(SqliteSamStore::builder()
            .contact_store(SqliteContactStore::new(database.clone()))
            .account_store(SqliteAccountStore::new(database.clone()))
            .message_store(SqliteMessageStore::new(database.clone(), 10))
            .build())
    }
}

#[cfg(test)]
mod test {
    use libsignal_protocol::IdentityKeyPair;
    use rand::rngs::OsRng;
    use sam_common::address::RegistrationId;
    use tempfile::NamedTempFile;

    use crate::storage::{SignalStoreConfig, SqliteSignalStoreConfig};

    #[tokio::test]
    async fn sqlite_database_file_can_be_created() {
        let mut csprng = OsRng;
        let temp = NamedTempFile::new().expect("Can create tempfile");
        let path = format!("sqlite://{}?mode=rwc", temp.path().to_string_lossy());
        let store = SqliteSignalStoreConfig::new(path);
        let key_pair = IdentityKeyPair::generate(&mut csprng);
        let registration_id = RegistrationId::generate(&mut csprng);
        assert!(store.create_store(key_pair, registration_id).await.is_ok());
    }

    #[tokio::test]
    async fn sqlite_in_memory_database_can_be_created() {
        let mut csprng = OsRng;
        let store = SqliteSignalStoreConfig::new("sqlite::memory:".to_owned());
        let key_pair = IdentityKeyPair::generate(&mut csprng);
        let registration_id = RegistrationId::generate(&mut csprng);
        assert!(store.create_store(key_pair, registration_id).await.is_ok());
    }

    #[tokio::test]
    async fn sqlite_store_can_be_loaded() {
        let mut csprng = OsRng;
        let temp = NamedTempFile::new().expect("Can create tempfile");
        let path = format!("sqlite://{}?mode=rwc", temp.path().to_string_lossy());
        let config = SqliteSignalStoreConfig::new(path.clone());
        let key_pair = IdentityKeyPair::generate(&mut csprng);
        let registration_id = RegistrationId::generate(&mut csprng);
        let store = config
            .create_store(key_pair, registration_id)
            .await
            .expect("Can create a sqlite store");

        drop(store);

        assert!(SqliteSignalStoreConfig::new(path).load().await.is_ok());
    }
}
