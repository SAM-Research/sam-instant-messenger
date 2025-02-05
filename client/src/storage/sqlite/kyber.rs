use async_trait::async_trait;
use base64::{prelude::BASE64_STANDARD, Engine as _};
use libsignal_protocol::{
    GenericSignedPreKey as _, KyberPreKeyId, KyberPreKeyRecord, KyberPreKeyStore,
    SignalProtocolError,
};
use sqlx::{Pool, Sqlite};

use crate::{storage::ProvidesKeyId, ClientError};

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
impl ProvidesKeyId for SqliteKyberPreKeyStore {
    type KeyIdType = KyberPreKeyId;

    async fn next_key_id(&self) -> Result<Self::KeyIdType, ClientError> {
        todo!()
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
/*

    #[tokio::test]
    async fn get_and_save_kyber_pre_key_test() {
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
        let mut device_kyber_pre_key_store = DeviceKyberPreKeyStore::new(device);
        let kyber_pre_key_record = key_man
            .generate_kyber_pre_key(
                &mut device_identity_key_store,
                &mut device_kyber_pre_key_store,
            )
            .await
            .unwrap();

        device_kyber_pre_key_store
            .save_kyber_pre_key(kyber_pre_key_record.id().unwrap(), &kyber_pre_key_record)
            .await
            .unwrap();

        let retrived_record = device_kyber_pre_key_store
            .get_kyber_pre_key(kyber_pre_key_record.id().unwrap())
            .await
            .unwrap();

        assert_eq!(
            retrived_record.id().unwrap(),
            kyber_pre_key_record.id().unwrap()
        );

        assert_eq!(
            retrived_record.public_key().unwrap().serialize(),
            kyber_pre_key_record
                .key_pair()
                .unwrap()
                .public_key
                .serialize()
        );

        assert_eq!(
            retrived_record.secret_key().unwrap().serialize(),
            kyber_pre_key_record
                .key_pair()
                .unwrap()
                .secret_key
                .serialize()
        );
    }


*/
