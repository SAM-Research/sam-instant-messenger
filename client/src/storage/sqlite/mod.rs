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

use super::{Store, StoreConfig, StoreType};
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

    type MessageStore = SqliteMessageStore;
}

pub type SqliteStore = Store<SqliteStoreType>;
#[derive(Debug)]
pub struct SqliteStoreConfig {
    connection_string: String,
}

impl SqliteStoreConfig {
    pub fn new(connection_string: String) -> Self {
        Self { connection_string }
    }

    pub async fn in_memory() -> Self {
        Self {
            connection_string: "sqlite::memory:".to_owned(),
        }
    }

    /// Load an existing database.
    pub async fn load(self) -> Result<SqliteStore, ClientError> {
        let database = SqlitePoolOptions::new()
            .connect(&self.connection_string)
            .await
            .map_err(|err| {
                ClientError::Database(format!(
                    "Could not connect to the database at '{}': {}",
                    self.connection_string, err
                ))
            })?;

        Ok(SqliteStore::builder()
            .contact_store(SqliteContactStore::new(database.clone()))
            .account_store(SqliteAccountStore::new(database.clone()))
            .pre_key_store(SqlitePreKeyStore::new(database.clone()))
            .signed_pre_key_store(SqliteSignedPreKeyStore::new(database.clone()))
            .kyber_pre_key_store(SqliteKyberPreKeyStore::new(database.clone()))
            .sender_key_store(SqliteSenderKeyStore::new(database.clone()))
            .session_store(SqliteSessionStore::new(database.clone()))
            .message_store(SqliteMessageStore::new(database.clone(), 10))
            .identity_key_store(SqliteIdentityKeyStore::load(database.clone()).await?)
            .build())
    }
}

#[async_trait(?Send)]
impl StoreConfig for SqliteStoreConfig {
    type StoreType = SqliteStoreType;

    /// Create a new database and run migrations.
    async fn create_store<ID: Into<u32>>(
        self,
        key_pair: IdentityKeyPair,
        registration_id: ID,
    ) -> Result<SqliteStore, ClientError> {
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

        Ok(SqliteStore::builder()
            .contact_store(SqliteContactStore::new(database.clone()))
            .account_store(SqliteAccountStore::new(database.clone()))
            .pre_key_store(SqlitePreKeyStore::new(database.clone()))
            .signed_pre_key_store(SqliteSignedPreKeyStore::new(database.clone()))
            .kyber_pre_key_store(SqliteKyberPreKeyStore::new(database.clone()))
            .sender_key_store(SqliteSenderKeyStore::new(database.clone()))
            .session_store(SqliteSessionStore::new(database.clone()))
            .message_store(SqliteMessageStore::new(database.clone(), 10))
            .identity_key_store(
                SqliteIdentityKeyStore::new(database.clone(), key_pair, registration_id.into())
                    .await?,
            )
            .build())
    }
}

#[cfg(test)]
mod test {
    use libsignal_protocol::IdentityKeyPair;
    use rand::rngs::OsRng;
    use sam_common::address::RegistrationId;
    use tempfile::NamedTempFile;

    use crate::storage::{SqliteStoreConfig, StoreConfig};

    #[tokio::test]
    async fn sqlite_database_file_can_be_created() {
        let mut csprng = OsRng;
        let temp = NamedTempFile::new().expect("Can create tempfile");
        let path = format!("sqlite://{}?mode=rwc", temp.path().to_string_lossy());
        let store = SqliteStoreConfig::new(path);
        let key_pair = IdentityKeyPair::generate(&mut csprng);
        let registration_id = RegistrationId::generate(&mut csprng);
        assert!(store.create_store(key_pair, registration_id).await.is_ok());
    }

    #[tokio::test]
    async fn sqlite_in_memory_database_can_be_created() {
        let mut csprng = OsRng;
        let store = SqliteStoreConfig::new("sqlite::memory:".to_owned());
        let key_pair = IdentityKeyPair::generate(&mut csprng);
        let registration_id = RegistrationId::generate(&mut csprng);
        assert!(store.create_store(key_pair, registration_id).await.is_ok());
    }

    #[tokio::test]
    async fn sqlite_store_can_be_loaded() {
        let mut csprng = OsRng;
        let temp = NamedTempFile::new().expect("Can create tempfile");
        let path = format!("sqlite://{}?mode=rwc", temp.path().to_string_lossy());
        let config = SqliteStoreConfig::new(path.clone());
        let key_pair = IdentityKeyPair::generate(&mut csprng);
        let registration_id = RegistrationId::generate(&mut csprng);
        let store = config
            .create_store(key_pair, registration_id)
            .await
            .expect("Can create a sqlite store");

        drop(store);

        assert!(SqliteStoreConfig::new(path).load().await.is_ok());
    }
}
