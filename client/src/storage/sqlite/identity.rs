use async_trait::async_trait;
use base64::{prelude::BASE64_STANDARD, Engine as _};
use libsignal_protocol::{
    Direction, IdentityKey, IdentityKeyPair, IdentityKeyStore, PrivateKey, ProtocolAddress,
    SignalProtocolError,
};
use sqlx::{Error as SqlxError, Pool, Sqlite};

use crate::{storage::error::DatabaseError, ClientError};

#[derive(Debug)]
pub struct SqliteIdentityKeyStore {
    database: Pool<Sqlite>,
}

impl SqliteIdentityKeyStore {
    pub async fn insert_identity(
        &self,
        address: &ProtocolAddress,
        identity: &IdentityKey,
    ) -> Result<(), SqlxError> {
        let addr = format!("{}", address);
        let key = BASE64_STANDARD.encode(identity.serialize());

        sqlx::query!(
            r#"
            INSERT INTO DeviceIdentityKeyStore (address, identity_key)
            VALUES (?, ?)
            ON CONFLICT(address) DO UPDATE SET identity_key = ?
            "#,
            addr,
            key,
            key
        )
        .execute(&self.database)
        .await
        .map(|_| ())
    }

    async fn insert_account_key_information(
        &self,
        key_pair: IdentityKeyPair,
        registration_id: u32,
    ) -> Result<(), ClientError> {
        let pk = BASE64_STANDARD.encode(key_pair.identity_key().serialize());
        let sk = BASE64_STANDARD.encode(key_pair.private_key().serialize());

        sqlx::query!(
            r#"
            INSERT INTO IdentityKeys (public_key, private_key, registration_id)
            VALUES (?, ?, ?)
            "#,
            pk,
            sk,
            registration_id,
        )
        .execute(&self.database)
        .await
        .map(|_| ())
        .map_err(ClientError::from)
    }

    pub async fn create(
        database: Pool<Sqlite>,
        key_pair: IdentityKeyPair,
        registration_id: u32,
    ) -> Result<Self, ClientError> {
        let id_store = Self { database };
        id_store
            .insert_account_key_information(key_pair, registration_id)
            .await?;
        Ok(id_store)
    }

    pub async fn load(database: Pool<Sqlite>) -> Result<Self, ClientError> {
        Ok(Self { database })
    }
}

#[async_trait(?Send)]
impl IdentityKeyStore for SqliteIdentityKeyStore {
    async fn get_identity_key_pair(&self) -> Result<IdentityKeyPair, SignalProtocolError> {
        match sqlx::query!(
            r#"
            SELECT
                public_key, private_key
            FROM
                IdentityKeys 
            "#
        )
        .fetch_one(&self.database)
        .await
        {
            Ok(row) => Ok(IdentityKeyPair::new(
                IdentityKey::decode(&BASE64_STANDARD.decode(row.public_key).map_err(|err| {
                    SignalProtocolError::ApplicationCallbackError(
                        "Could not decode base64 Identity Key from database into bytes",
                        Box::new(err),
                    )
                })?)
                .map_err(|err| {
                    SignalProtocolError::ApplicationCallbackError(
                        "Could not decode bytes into Identity Key",
                        Box::new(err),
                    )
                })?,
                PrivateKey::deserialize(&BASE64_STANDARD.decode(row.private_key).map_err(
                    |err| {
                        SignalProtocolError::ApplicationCallbackError(
                            "Could not decode base64 Identity Private Key from database into bytes",
                            Box::new(err),
                        )
                    },
                )?)
                .map_err(|err| {
                    SignalProtocolError::ApplicationCallbackError(
                        "Could not decode bytes into Identity Private Key",
                        Box::new(err),
                    )
                })?,
            )),
            Err(err) => Err(SignalProtocolError::ApplicationCallbackError(
                "Could not fetch Identity Key bundle from database",
                Box::new(DatabaseError::from(err)),
            )),
        }
    }

    async fn get_local_registration_id(&self) -> Result<u32, SignalProtocolError> {
        sqlx::query!(
            r#"
            SELECT
                registration_id
            FROM
                IdentityKeys
            "#
        )
        .fetch_one(&self.database)
        .await
        .map(|row| row.registration_id as u32)
        .map_err(|err| {
            SignalProtocolError::ApplicationCallbackError(
                "Could not Retrieve local registration id",
                Box::new(DatabaseError::from(err)),
            )
        })
    }

    async fn save_identity(
        &mut self,
        address: &ProtocolAddress,
        identity: &IdentityKey,
    ) -> Result<bool, SignalProtocolError> {
        match self
            .get_identity(address)
            .await
            .map_err(|err| SignalProtocolError::InvalidArgument(format!("{err}")))?
        {
            Some(key) if key == *identity => Ok(false),
            Some(_key) => {
                self.insert_identity(address, identity)
                    .await
                    .map_err(|err| {
                        SignalProtocolError::ApplicationCallbackError(
                            "Could not update own Identity",
                            Box::new(DatabaseError::from(err)),
                        )
                    })?;
                Ok(false)
            }
            None => {
                self.insert_identity(address, identity)
                    .await
                    .map_err(|err| {
                        SignalProtocolError::ApplicationCallbackError(
                            "Could not insert own Identity",
                            Box::new(DatabaseError::from(err)),
                        )
                    })?;
                Ok(true)
            }
        }
    }

    async fn is_trusted_identity(
        &self,
        address: &ProtocolAddress,
        identity: &IdentityKey,
        _direction: Direction,
    ) -> Result<bool, SignalProtocolError> {
        match self.get_identity(address).await? {
            Some(i) => Ok(i == *identity),
            None => Ok(true),
        }
    }

    async fn get_identity(
        &self,
        address: &ProtocolAddress,
    ) -> Result<Option<IdentityKey>, SignalProtocolError> {
        let addr = format!("{}", address);

        match sqlx::query!(
            r#"
            SELECT
                identity_key
            FROM
                DeviceIdentityKeyStore
            WHERE
                address = ?
            "#,
            addr
        )
        .fetch_one(&self.database)
        .await
        {
            Ok(row) => Ok(Some(
                BASE64_STANDARD
                    .decode(row.identity_key)
                    .map_err(|err| {
                        SignalProtocolError::ApplicationCallbackError(
                            "decode identity key from base64",
                            Box::new(err),
                        )
                    })?
                    .as_slice()
                    .try_into()?,
            )),
            Err(_) => Ok(None),
        }
    }
}

#[cfg(test)]
mod test {
    use libsignal_protocol::{Direction, IdentityKeyPair, IdentityKeyStore, ProtocolAddress};
    use rand::rngs::OsRng;

    use crate::storage::sqlite::sqlite_test::{bob_address, connect, key_pair};

    use super::SqliteIdentityKeyStore;

    async fn id_key_store(alice_key_pair: IdentityKeyPair) -> SqliteIdentityKeyStore {
        SqliteIdentityKeyStore::create(connect().await, alice_key_pair, 1)
            .await
            .expect("can create an identity key store with in memory test database")
    }

    /// Test no other identity exists in new store
    #[tokio::test]
    async fn no_identity_in_new_store() {
        let alice_key_pair = key_pair();
        let identity_key_store = id_key_store(alice_key_pair).await;

        assert_eq!(
            identity_key_store
                .get_identity(&bob_address())
                .await
                .expect("should be able to retrieve None"),
            None
        );
    }

    /// Test own identity exists in new store
    #[tokio::test]
    async fn own_identity_in_new_store() {
        let alice_key_pair = key_pair();
        let identity_key_store = id_key_store(alice_key_pair).await;

        assert_eq!(
            identity_key_store
                .get_identity_key_pair()
                .await
                .expect("should be able to retrieve own ID key pair")
                .public_key(),
            alice_key_pair.public_key()
        );
    }

    /// Test that a new identity have been added
    #[tokio::test]
    async fn save_identity() {
        let alice_key_pair = key_pair();
        let mut identity_key_store = id_key_store(alice_key_pair).await;
        let bob_address = ProtocolAddress::new("bob".to_owned(), 0.into());
        let bob_key_pair = IdentityKeyPair::generate(&mut OsRng);

        assert!(identity_key_store
            .save_identity(&bob_address, bob_key_pair.identity_key())
            .await
            .expect("sqlite identity key store can save an identity key"));

        assert_eq!(
            identity_key_store
                .get_identity(&bob_address)
                .await
                .expect("sqlite identity key store can retrive a saved identity key")
                .expect("the retrived identity key should exist"),
            *bob_key_pair.identity_key()
        );
    }

    #[tokio::test]
    async fn save_identity_returns_false_if_key_is_known() {
        let alice_key_pair = key_pair();
        let mut identity_key_store = id_key_store(alice_key_pair).await;
        let bob_address = ProtocolAddress::new("bob".to_owned(), 0.into());
        let bob_key_pair = IdentityKeyPair::generate(&mut OsRng);

        assert!(identity_key_store
            .save_identity(&bob_address, bob_key_pair.identity_key())
            .await
            .expect("sqlite identity key store can save an identity key"));

        assert!(!identity_key_store
            .save_identity(&bob_address, bob_key_pair.identity_key())
            .await
            .expect("sqlite identity key store can attempt to store identity key"));

        assert_eq!(
            identity_key_store
                .get_identity(&bob_address)
                .await
                .expect("sqlite identity key store can retrive a saved identity key")
                .expect("the retrived identity key should exist"),
            *bob_key_pair.identity_key()
        );
    }

    #[tokio::test]
    async fn save_identity_returns_false_if_key_overwritten() {
        let alice_key_pair = key_pair();
        let mut identity_key_store = id_key_store(alice_key_pair).await;
        let bob_address = ProtocolAddress::new("bob".to_owned(), 0.into());
        let bob_key_pair = IdentityKeyPair::generate(&mut OsRng);

        assert!(identity_key_store
            .save_identity(&bob_address, bob_key_pair.identity_key())
            .await
            .expect("sqlite identity key store can attempt to store identity key"));

        assert!(!identity_key_store
            .save_identity(&bob_address, alice_key_pair.identity_key())
            .await
            .expect("sqlite identity key store can attempt to store identity key"));

        assert_eq!(
            identity_key_store
                .get_identity(&bob_address)
                .await
                .expect("sqlite identity key store can retrive a saved identity key")
                .expect("the retrived identity key should exist"),
            *alice_key_pair.identity_key()
        );
    }

    #[tokio::test]
    async fn identity_is_trusted_on_first_use() {
        let alice_key_pair = key_pair();
        let mut identity_key_store = id_key_store(alice_key_pair).await;
        let bob_address = bob_address();
        let bob_key_pair = key_pair();
        // First use
        assert!(identity_key_store
            .is_trusted_identity(
                &bob_address,
                bob_key_pair.identity_key(),
                Direction::Sending
            )
            .await
            .unwrap());

        // Added identity
        identity_key_store
            .save_identity(&bob_address, bob_key_pair.identity_key())
            .await
            .unwrap();

        assert!(identity_key_store
            .is_trusted_identity(
                &bob_address,
                bob_key_pair.identity_key(),
                Direction::Sending
            )
            .await
            .unwrap());
    }

    #[tokio::test]
    async fn identity_is_not_trusted_before_first_use() {
        let alice_key_pair = key_pair();
        let mut identity_key_store = id_key_store(alice_key_pair).await;
        let bob_address = bob_address();
        let bob_key_pair = key_pair();
        let charlie_address = ProtocolAddress::new("charlie".to_owned(), 0.into());
        let charlie_key_pair = key_pair();

        // Added identity
        identity_key_store
            .save_identity(&bob_address, bob_key_pair.identity_key())
            .await
            .unwrap();

        // Not trusted
        identity_key_store
            .save_identity(&charlie_address, charlie_key_pair.identity_key())
            .await
            .unwrap();

        assert!(!identity_key_store
            .is_trusted_identity(
                &bob_address,
                charlie_key_pair.identity_key(),
                Direction::Sending
            )
            .await
            .unwrap());
    }
}
