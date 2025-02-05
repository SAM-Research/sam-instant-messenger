use async_trait::async_trait;
use base64::{prelude::BASE64_STANDARD, Engine as _};
use libsignal_protocol::{
    GenericSignedPreKey as _, SignalProtocolError, SignedPreKeyId, SignedPreKeyRecord,
    SignedPreKeyStore,
};
use sqlx::{Pool, Sqlite};

use crate::{storage::ProvidesKeyId, ClientError};

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
impl ProvidesKeyId for SqliteSignedPreKeyStore {
    type KeyIdType = SignedPreKeyId;

    async fn next_key_id(&self) -> Result<Self::KeyIdType, ClientError> {
        todo!()
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
/*
    #[tokio::test]
    async fn get_and_save_signed_pre_key_test() {
        let pool = connect().await;

        let device = Device::new(pool.clone());

        device
            .insert_account_key_information(
                IdentityKeyPair::generate(&mut OsRng),
                new_rand_number(),
            )
            .await
            .unwrap();

        let mut key_man = KeyManager::default();
        let mut device_identity_key_store = DeviceIdentityKeyStore::new(device.clone());
        let mut device_signed_pre_key_store = DeviceSignedPreKeyStore::new(device);
        let signed_pre_key_record = key_man
            .generate_signed_pre_key(
                &mut device_identity_key_store,
                &mut device_signed_pre_key_store,
                &mut OsRng,
            )
            .await
            .unwrap();
        device_signed_pre_key_store
            .save_signed_pre_key(signed_pre_key_record.id().unwrap(), &signed_pre_key_record)
            .await
            .unwrap();

        let retrived_record = device_signed_pre_key_store
            .get_signed_pre_key(signed_pre_key_record.id().unwrap())
            .await
            .unwrap();

        assert_eq!(
            retrived_record.id().unwrap(),
            signed_pre_key_record.id().unwrap()
        );
        assert_eq!(
            retrived_record.public_key().unwrap(),
            signed_pre_key_record.key_pair().unwrap().public_key
        );
        assert_eq!(
            retrived_record.private_key().unwrap().serialize(),
            signed_pre_key_record
                .key_pair()
                .unwrap()
                .private_key
                .serialize()
        );
    }
*/
