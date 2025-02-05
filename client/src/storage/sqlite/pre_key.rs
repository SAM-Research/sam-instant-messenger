use async_trait::async_trait;
use base64::{prelude::BASE64_STANDARD, Engine};
use libsignal_protocol::{PreKeyId, PreKeyRecord, PreKeyStore, SignalProtocolError};
use sqlx::{Pool, Sqlite};

use crate::{storage::ProvidesKeyId, ClientError};

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
impl ProvidesKeyId for SqlitePreKeyStore {
    type KeyIdType = PreKeyId;

    async fn next_key_id(&self) -> Result<Self::KeyIdType, ClientError> {
        sqlx::query!(
            r#"
            WITH max_pre_key_id_table AS (
                SELECT
                    1 AS _id,
                    MAX(id) AS max_pre_key_id
                FROM
                    DeviceSignedPreKeyStore
                )
                SELECT
                    CASE WHEN pk.max_pre_key_id IS NOT NULL
                    THEN pk.max_pre_key_id
                    ELSE
                    0
                    END AS pkid
                FROM
                    max_pre_key_id_table pk
                "#
        )
        .fetch_one(&self.database)
        .await
        .map(|row| Self::KeyIdType::from(row.pkid as u32))
        .map_err(|err| ClientError::from(err))
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

#[cfg(test)]
mod test {
    #[tokio::test]
    async fn no_identity_in_new_store() {}
}
/*
#[tokio::test]
    async fn save_and_get_pre_key_test() {
        let mut key_man = KeyManager::default();
        let device = Device::new(connect().await);
        let mut device_pre_key_store = DevicePreKeyStore::new(device);
        let pre_key_record = key_man
            .generate_pre_key(&mut device_pre_key_store, &mut OsRng)
            .await
            .unwrap();

        device_pre_key_store
            .save_pre_key(pre_key_record.id().unwrap(), &pre_key_record)
            .await
            .unwrap();

        let retrived_pre_key = device_pre_key_store
            .get_pre_key(pre_key_record.id().unwrap())
            .await
            .unwrap();

        assert_eq!(retrived_pre_key.id().unwrap(), pre_key_record.id().unwrap());

        assert_eq!(
            retrived_pre_key.public_key().unwrap(),
            pre_key_record.key_pair().unwrap().public_key
        );

        assert_eq!(
            retrived_pre_key.private_key().unwrap().serialize(),
            pre_key_record.key_pair().unwrap().private_key.serialize()
        );
    }
    #[tokio::test]
    async fn remove_pre_key_test() {
        let mut key_man = KeyManager::default();
        let device = Device::new(connect().await);
        let mut device_pre_key_store = DevicePreKeyStore::new(device);
        let pre_key_record = key_man
            .generate_pre_key(&mut device_pre_key_store, &mut OsRng)
            .await
            .unwrap();

        device_pre_key_store
            .save_pre_key(pre_key_record.id().unwrap(), &pre_key_record)
            .await
            .unwrap();

        let _ = device_pre_key_store
            .get_pre_key(pre_key_record.id().unwrap())
            .await
            .unwrap();

        device_pre_key_store
            .remove_pre_key(pre_key_record.id().unwrap())
            .await
            .unwrap();

        device_pre_key_store
            .get_pre_key(pre_key_record.id().unwrap())
            .await
            .expect_err("We should not be able to retrive the key after deletion");
    }
*/
