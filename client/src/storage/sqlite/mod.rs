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
use sqlite_connector::SqliteConnector;

use super::{error::StoreCreationError, Store, StoreConfig, StoreType};

pub mod account;
pub mod contact;
pub mod identity;
pub mod kyber;
pub mod message;
pub mod pre_key;
pub mod sender_key;
pub mod session;
pub mod signed_pre_key;
pub mod sqlite_connector;

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

pub struct SqliteStoreConfig {
    buffer_size: usize,
    connector: SqliteConnector,
}

impl SqliteStoreConfig {
    pub fn new(connector: SqliteConnector, buffer_size: usize) -> Self {
        Self {
            buffer_size,
            connector,
        }
    }

    pub async fn in_memory(buffer_size: usize) -> Result<Self, StoreCreationError> {
        let connector = SqliteConnector::migrate("sqlite::memory:").await?;
        Ok(Self {
            buffer_size,
            connector,
        })
    }
}

impl Into<SqliteStore> for SqliteStoreConfig {
    fn into(self) -> SqliteStore {
        SqliteStore::builder()
            .contact_store(SqliteContactStore::new(self.connector.pool()))
            .account_store(SqliteAccountStore::new(self.connector.pool()))
            .pre_key_store(SqlitePreKeyStore::new(self.connector.pool()))
            .signed_pre_key_store(SqliteSignedPreKeyStore::new(self.connector.pool()))
            .kyber_pre_key_store(SqliteKyberPreKeyStore::new(self.connector.pool()))
            .sender_key_store(SqliteSenderKeyStore::new(self.connector.pool()))
            .session_store(SqliteSessionStore::new(self.connector.pool()))
            .message_store(SqliteMessageStore::new(
                self.connector.pool(),
                self.buffer_size,
            ))
            .identity_key_store(SqliteIdentityKeyStore::load(self.connector.into()))
            .build()
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
    ) -> Result<SqliteStore, StoreCreationError> {
        Ok(SqliteStore::builder()
            .contact_store(SqliteContactStore::new(self.connector.pool()))
            .account_store(SqliteAccountStore::new(self.connector.pool()))
            .pre_key_store(SqlitePreKeyStore::new(self.connector.pool()))
            .signed_pre_key_store(SqliteSignedPreKeyStore::new(self.connector.pool()))
            .kyber_pre_key_store(SqliteKyberPreKeyStore::new(self.connector.pool()))
            .sender_key_store(SqliteSenderKeyStore::new(self.connector.pool()))
            .session_store(SqliteSessionStore::new(self.connector.pool()))
            .message_store(SqliteMessageStore::new(
                self.connector.pool(),
                self.buffer_size,
            ))
            .identity_key_store(
                SqliteIdentityKeyStore::new(
                    self.connector.into(),
                    key_pair,
                    registration_id.into(),
                )
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

    use crate::storage::{
        sqlite::sqlite_connector::SqliteConnector, SqliteStoreConfig, StoreConfig,
    };

    #[tokio::test]
    async fn sqlite_database_file_can_be_created() {
        let mut csprng = OsRng;
        let temp = NamedTempFile::new().expect("Can create tempfile");
        let path = format!("sqlite://{}?mode=rwc", temp.path().to_string_lossy());
        let connector = SqliteConnector::migrate(&path)
            .await
            .expect("can create connector");
        let store = SqliteStoreConfig::new(connector, 10);
        let key_pair = IdentityKeyPair::generate(&mut csprng);
        let registration_id = RegistrationId::generate(&mut csprng);
        assert!(store.create_store(key_pair, registration_id).await.is_ok());
    }

    #[tokio::test]
    async fn sqlite_in_memory_database_can_be_created() {
        let mut csprng = OsRng;
        let store = SqliteStoreConfig::in_memory(10)
            .await
            .expect("can create inmemory");
        let key_pair = IdentityKeyPair::generate(&mut csprng);
        let registration_id = RegistrationId::generate(&mut csprng);
        assert!(store.create_store(key_pair, registration_id).await.is_ok());
    }

    #[tokio::test]
    async fn sqlite_store_can_be_loaded() {
        let mut csprng = OsRng;
        let temp = NamedTempFile::new().expect("Can create tempfile");
        let path = format!("sqlite://{}?mode=rwc", temp.path().to_string_lossy());
        let connector = SqliteConnector::migrate(&path)
            .await
            .expect("can create connector");
        let config = SqliteStoreConfig::new(connector, 10);
        let key_pair = IdentityKeyPair::generate(&mut csprng);
        let registration_id = RegistrationId::generate(&mut csprng);
        let store = config
            .create_store(key_pair, registration_id)
            .await
            .expect("Can create a sqlite store");

        drop(store);

        assert!(SqliteConnector::connect(&path).await.is_ok());
    }
}
