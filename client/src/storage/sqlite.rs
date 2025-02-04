use crate::ClientError;
use async_trait::async_trait;
use base64::{prelude::BASE64_STANDARD, Engine as _};
use libsignal_protocol::{
    Aci, Direction, GenericSignedPreKey as _, IdentityKey, IdentityKeyPair, IdentityKeyStore,
    KyberPreKeyId, KyberPreKeyRecord, KyberPreKeyStore, Pni, PreKeyId, PreKeyRecord, PreKeyStore,
    PrivateKey, ProtocolAddress, SenderKeyRecord, SenderKeyStore, SessionRecord, SessionStore,
    SignalProtocolError, SignedPreKeyId, SignedPreKeyRecord, SignedPreKeyStore,
};
use sqlx::{Pool, Sqlite};
use uuid::Uuid;

use super::{AccountStore, ContactStore, Store, StoreConfig, StoreType};

#[derive(Debug)]
pub struct SqliteStoreConfig {
    database: Pool<Sqlite>,
}

#[async_trait(?Send)]
impl StoreConfig for SqliteStoreConfig {
    type StoreType = SqliteStoreType;
    async fn build<ID: Into<u32>>(
        self,
        key_pair: IdentityKeyPair,
        registration_id: ID,
    ) -> crate::Result<SqliteStore> {
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
}

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
pub struct SqliteContactStore {
    _database: Pool<Sqlite>,
}

impl SqliteContactStore {
    pub fn new(database: Pool<Sqlite>) -> Self {
        Self {
            _database: database,
        }
    }
}

impl ContactStore for SqliteContactStore {}

impl SqliteAccountStore {
    pub fn new(database: Pool<Sqlite>) -> Self {
        Self {
            _database: database,
        }
    }
}

#[derive(Debug)]
pub struct SqliteAccountStore {
    _database: Pool<Sqlite>,
}

impl AccountStore for SqliteAccountStore {
    fn set_aci(&self, _aci: Aci) -> crate::Result<()> {
        todo!()
    }

    fn get_aci(&self) -> crate::Result<Aci> {
        todo!()
    }

    fn set_pni(&self, _pni: Pni) -> crate::Result<()> {
        todo!()
    }

    fn set_password(&self, _password: String) -> crate::Result<()> {
        todo!()
    }
}

#[derive(Debug)]
pub struct SqliteIdentityKeyStore {
    database: Pool<Sqlite>,
}

impl SqliteIdentityKeyStore {
    pub async fn insert_identity(
        &self,
        address: &ProtocolAddress,
        identity: &IdentityKey,
    ) -> crate::Result<()> {
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
        .map_err(|err| ClientError::from(err))
    }
}

impl SqliteIdentityKeyStore {
    pub fn new(database: Pool<Sqlite>, _key_pair: IdentityKeyPair, _registration_id: u32) -> Self {
        Self { database }
    }
}

#[async_trait(?Send)]
impl IdentityKeyStore for SqliteIdentityKeyStore {
    async fn get_identity_key_pair(
        &self,
    ) -> std::result::Result<IdentityKeyPair, SignalProtocolError> {
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
                Box::new(ClientError::from(err)),
            )),
        }
    }

    async fn get_local_registration_id(&self) -> std::result::Result<u32, SignalProtocolError> {
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
                Box::new(ClientError::from(err)),
            )
        })
    }

    async fn save_identity(
        &mut self,
        address: &ProtocolAddress,
        identity: &IdentityKey,
    ) -> std::result::Result<bool, SignalProtocolError> {
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
                            Box::new(err),
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
                            Box::new(err),
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
    ) -> std::result::Result<bool, SignalProtocolError> {
        match self.get_identity(address).await? {
            Some(i) => Ok(i == *identity),
            None => Ok(true),
        }
    }

    async fn get_identity(
        &self,
        address: &ProtocolAddress,
    ) -> std::result::Result<Option<IdentityKey>, SignalProtocolError> {
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

#[derive(Debug)]
pub struct SqlitePreKeyStore {
    database: Pool<Sqlite>,
}

impl SqlitePreKeyStore {
    pub fn new(database: Pool<Sqlite>) -> Self {
        Self { database }
    }
}

#[async_trait(?Send)]
impl PreKeyStore for SqlitePreKeyStore {
    async fn get_pre_key(&self, prekey_id: PreKeyId) -> Result<PreKeyRecord, SignalProtocolError> {
        let id: u32 = prekey_id.into();

        match sqlx::query!(
            r#"
            SELECT
                pre_key_record
            FROM
                DevicePreKeyStore
            WHERE
                pre_key_id = ?
            "#,
            id
        )
        .fetch_one(&self.database)
        .await
        {
            Ok(row) => PreKeyRecord::deserialize(
                BASE64_STANDARD
                    .decode(row.pre_key_record)
                    .map_err(|err| {
                        SignalProtocolError::ApplicationCallbackError(
                            "decode pre key from base64",
                            Box::new(err),
                        )
                    })?
                    .as_slice(),
            )
            .map_err(|err| {
                SignalProtocolError::ApplicationCallbackError(
                    "deserialize pre key from bytes",
                    Box::new(err),
                )
            }),
            Err(err) => Err(SignalProtocolError::ApplicationCallbackError(
                "save pre key",
                Box::new(ClientError::from(err)),
            )),
        }
    }

    async fn save_pre_key(
        &mut self,
        prekey_id: PreKeyId,
        record: &PreKeyRecord,
    ) -> Result<(), SignalProtocolError> {
        let id: u32 = prekey_id.into();
        let rec = BASE64_STANDARD.encode(record.serialize()?);

        sqlx::query!(
            r#"
            INSERT INTO DevicePreKeyStore (pre_key_id, pre_key_record)
            VALUES (?, ?)
            ON CONFLICT(pre_key_id) DO UPDATE SET pre_key_record = ?
            "#,
            id,
            rec,
            rec
        )
        .execute(&self.database)
        .await
        .map(|_| ())
        .map_err(|err| {
            SignalProtocolError::ApplicationCallbackError(
                "save pre key",
                Box::new(ClientError::from(err)),
            )
        })
    }

    async fn remove_pre_key(&mut self, prekey_id: PreKeyId) -> Result<(), SignalProtocolError> {
        let id: u32 = prekey_id.into();

        sqlx::query!(
            r#"
            DELETE FROM
                DevicePreKeyStore
            WHERE
                pre_key_id = ?
            "#,
            id
        )
        .execute(&self.database)
        .await
        .map(|_| ())
        .map_err(|err| {
            SignalProtocolError::ApplicationCallbackError(
                "remove pre key",
                Box::new(ClientError::from(err)),
            )
        })
    }
}

#[derive(Debug)]
pub struct SqliteSignedPreKeyStore {
    database: Pool<Sqlite>,
}

impl SqliteSignedPreKeyStore {
    pub fn new(database: Pool<Sqlite>) -> Self {
        Self { database }
    }
}

#[async_trait(?Send)]
impl SignedPreKeyStore for SqliteSignedPreKeyStore {
    async fn get_signed_pre_key(
        &self,
        id: SignedPreKeyId,
    ) -> Result<SignedPreKeyRecord, SignalProtocolError> {
        let sid: u32 = id.into();

        match sqlx::query!(
            r#"
            SELECT
                signed_pre_key_record
            FROM
                DeviceSignedPreKeyStore
            WHERE
                signed_pre_key_id = ?
            "#,
            sid
        )
        .fetch_one(&self.database)
        .await
        {
            Ok(row) => SignedPreKeyRecord::deserialize(
                BASE64_STANDARD
                    .decode(row.signed_pre_key_record)
                    .map_err(|err| {
                        SignalProtocolError::ApplicationCallbackError(
                            "decode signed pre key",
                            Box::new(err),
                        )
                    })?
                    .as_slice(),
            )
            .map_err(|err| {
                SignalProtocolError::ApplicationCallbackError(
                    "deserialize signed pre key",
                    Box::new(err),
                )
            }),
            Err(err) => Err(SignalProtocolError::ApplicationCallbackError(
                "save signed pre key",
                Box::new(ClientError::from(err)),
            )),
        }
    }

    async fn save_signed_pre_key(
        &mut self,
        id: SignedPreKeyId,
        record: &SignedPreKeyRecord,
    ) -> Result<(), SignalProtocolError> {
        let id: u32 = id.into();
        let rec = BASE64_STANDARD.encode(record.serialize()?);

        sqlx::query!(
            r#"
            INSERT INTO DeviceSignedPreKeyStore (signed_pre_key_id, signed_pre_key_record)
            VALUES (?, ?)
            ON CONFLICT(signed_pre_key_id) DO UPDATE SET signed_pre_key_record = ?
            "#,
            id,
            rec,
            rec
        )
        .execute(&self.database)
        .await
        .map(|_| ())
        .map_err(|err| {
            SignalProtocolError::ApplicationCallbackError(
                "save signed pre key",
                Box::new(ClientError::from(err)),
            )
        })
    }
}

#[derive(Debug)]
pub struct SqliteKyberPreKeyStore {
    database: Pool<Sqlite>,
}

impl SqliteKyberPreKeyStore {
    pub fn new(database: Pool<Sqlite>) -> Self {
        Self { database }
    }
}

#[async_trait(?Send)]
impl KyberPreKeyStore for SqliteKyberPreKeyStore {
    async fn get_kyber_pre_key(
        &self,
        kyber_prekey_id: KyberPreKeyId,
    ) -> Result<KyberPreKeyRecord, SignalProtocolError> {
        let id: u32 = kyber_prekey_id.into();

        match sqlx::query!(
            r#"
            SELECT
                kyber_pre_key_record
            FROM
                DeviceKyberPreKeyStore
            WHERE
                kyber_pre_key_id = ?
            "#,
            id
        )
        .fetch_one(&self.database)
        .await
        {
            Ok(row) => KyberPreKeyRecord::deserialize(
                BASE64_STANDARD
                    .decode(row.kyber_pre_key_record)
                    .map_err(|err| {
                        SignalProtocolError::ApplicationCallbackError(
                            "get kyber pre key",
                            Box::new(err),
                        )
                    })?
                    .as_slice(),
            ),
            Err(_) => Err(SignalProtocolError::InvalidKyberPreKeyId),
        }
    }

    async fn save_kyber_pre_key(
        &mut self,
        kyber_prekey_id: KyberPreKeyId,
        record: &KyberPreKeyRecord,
    ) -> Result<(), SignalProtocolError> {
        let id: u32 = kyber_prekey_id.into();
        let rec = BASE64_STANDARD.encode(record.serialize()?);

        sqlx::query!(
            r#"
            INSERT INTO DeviceKyberPreKeyStore (kyber_pre_key_id, kyber_pre_key_record)
            VALUES (?, ?)
            ON CONFLICT(kyber_pre_key_id) DO UPDATE SET kyber_pre_key_record = ?
            "#,
            id,
            rec,
            rec
        )
        .execute(&self.database)
        .await
        .map(|_| ())
        .map_err(|err| {
            SignalProtocolError::ApplicationCallbackError(
                "save kyber pre key",
                Box::new(ClientError::from(err)),
            )
        })
    }

    async fn mark_kyber_pre_key_used(
        &mut self,
        _kyber_prekey_id: KyberPreKeyId,
    ) -> std::result::Result<(), SignalProtocolError> {
        // TODO: Why does Signal have this??
        Ok(())
    }
}

#[derive(Debug)]
pub struct SqliteSessionStore {
    database: Pool<Sqlite>,
}

impl SqliteSessionStore {
    pub fn new(database: Pool<Sqlite>) -> Self {
        Self { database }
    }
}

#[async_trait(?Send)]
impl SessionStore for SqliteSessionStore {
    async fn load_session(
        &self,
        address: &ProtocolAddress,
    ) -> Result<Option<SessionRecord>, SignalProtocolError> {
        let addr = format!("{}", address);

        match sqlx::query!(
            r#"
            SELECT
                session_record
            FROM
                DeviceSessionStore
            WHERE
                address = ?
            "#,
            addr
        )
        .fetch_one(&self.database)
        .await
        {
            Ok(row) => SessionRecord::deserialize(
                BASE64_STANDARD
                    .decode(row.session_record)
                    .map_err(|err| {
                        SignalProtocolError::ApplicationCallbackError(
                            "load session from database",
                            Box::new(err),
                        )
                    })?
                    .as_slice(),
            )
            .map(Some),
            Err(_) => Ok(None),
        }
    }
    async fn store_session(
        &mut self,
        address: &ProtocolAddress,
        record: &SessionRecord,
    ) -> Result<(), SignalProtocolError> {
        let addr = format!("{}", address);
        let rec = BASE64_STANDARD.encode(record.serialize()?);

        sqlx::query!(
            r#"
            INSERT INTO DeviceSessionStore (address, session_record)
            VALUES (?, ?)
            ON CONFLICT(address) DO UPDATE SET session_record = ?
            "#,
            addr,
            rec,
            rec
        )
        .execute(&self.database)
        .await
        .map(|_| ())
        .map_err(|err| {
            SignalProtocolError::ApplicationCallbackError(
                "store session",
                Box::new(ClientError::from(err)),
            )
        })
    }
}

#[derive(Debug)]
pub struct SqliteSenderKeyStore {
    database: Pool<Sqlite>,
}

impl SqliteSenderKeyStore {
    pub fn new(database: Pool<Sqlite>) -> Self {
        Self { database }
    }
}

#[async_trait(?Send)]
impl SenderKeyStore for SqliteSenderKeyStore {
    async fn store_sender_key(
        &mut self,
        sender: &ProtocolAddress,
        distribution_id: Uuid,
        record: &SenderKeyRecord,
    ) -> Result<(), SignalProtocolError> {
        let addr = format!("{}:{}", sender, distribution_id);
        let rec = BASE64_STANDARD.encode(record.serialize()?);

        sqlx::query!(
            r#"
            INSERT INTO DeviceSenderKeyStore (address, sender_key_record)
            VALUES (?, ?)
            ON CONFLICT(address) DO UPDATE SET sender_key_record = ?
            "#,
            addr,
            rec,
            rec
        )
        .execute(&self.database)
        .await
        .map(|_| ())
        .map_err(|err| {
            SignalProtocolError::ApplicationCallbackError(
                "store sender key",
                Box::new(ClientError::from(err)),
            )
        })
    }
    async fn load_sender_key(
        &mut self,
        sender: &ProtocolAddress,
        distribution_id: Uuid,
    ) -> Result<Option<SenderKeyRecord>, SignalProtocolError> {
        let addr = format!("{}:{}", sender, distribution_id);

        match sqlx::query!(
            r#"
            SELECT
                sender_key_record
            FROM
                DeviceSenderKeyStore
            WHERE
                address = ?
            "#,
            addr
        )
        .fetch_one(&self.database)
        .await
        {
            Ok(row) => SenderKeyRecord::deserialize(
                BASE64_STANDARD
                    .decode(row.sender_key_record)
                    .map_err(|err| {
                        SignalProtocolError::ApplicationCallbackError(
                            "load sender key",
                            Box::new(err),
                        )
                    })?
                    .as_slice(),
            )
            .map(Some),
            Err(_) => Ok(None), // TODO: Mistake?
        }
    }
}
