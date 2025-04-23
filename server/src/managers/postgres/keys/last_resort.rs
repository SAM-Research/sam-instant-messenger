use async_trait::async_trait;
use libsignal_protocol::IdentityKey;
use log::{debug, error};
use sam_common::{
    api::{Key as _, PqPreKey},
    AccountId, DeviceId,
};
use sqlx::{Pool, Postgres};

use crate::{
    auth::keys::verify_key,
    managers::{error::KeyManagerError, traits::key_manager::LastResortPqPreKeyManager},
};

#[derive(Debug, Clone)]
pub struct PostgresLastResortPqPreKeyManager {
    pool: Pool<Postgres>,
}

impl PostgresLastResortPqPreKeyManager {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl LastResortPqPreKeyManager for PostgresLastResortPqPreKeyManager {
    async fn get_last_resort_key(
        &self,
        account_id: AccountId,
        device_id: DeviceId,
    ) -> Result<PqPreKey, KeyManagerError> {
        let aci = account_id.uuid();
        let dev = *device_id as i64;
        match sqlx::query!(
            r#"
            SELECT key_id, public_key, signature
            FROM last_resort_pq_pre_keys
            WHERE owner =
                (SELECT id
                 FROM devices
                 WHERE owner = 
                         (SELECT id
                         FROM accounts
                         WHERE account_id = $1)
                   AND devices.device_id = $2)
            "#,
            aci,
            dev
        )
        .fetch_one(&self.pool)
        .await
        {
            Ok(row) => {
                let public_key = row.public_key.into();
                let signature = row.signature.into();
                let key_id = row
                    .key_id
                    .try_into()
                    .map_err(|_| {
                        error!("Device {device_id} belonging to {account_id} has a Last Resort PQ pre key with an invalid ID '{}' ", row.key_id);
                        KeyManagerError::ServiceUnavailable})?;
                Ok(PqPreKey {
                    public_key,
                    key_id,
                    signature,
                })
            }
            Err(sqlx::Error::RowNotFound) => Err(KeyManagerError::KeyDoesNotExist),
            Err(err) => {
                error!(
                    "Error while attempting to fetch a pre key for {account_id}.{device_id}: {err}"
                );
                Err(KeyManagerError::ServiceUnavailable)
            }
        }
    }

    async fn set_last_resort_key(
        &mut self,
        account_id: AccountId,
        device_id: DeviceId,
        identity: &IdentityKey,
        key: PqPreKey,
    ) -> Result<(), KeyManagerError> {
        let aci = account_id.uuid();
        let dev = *device_id as i64;
        let k_id = key.id() as i64;

        verify_key(identity, &key)?;

        let p_key = &*key.public_key;
        let sig = &*key.signature;
        match sqlx::query!(
            r#"
            INSERT INTO last_resort_pq_pre_keys (owner, key_id, public_key, signature)
            SELECT id,
                   $3,
                   $4,
                   $5
            FROM devices
            WHERE owner = 
                (SELECT id
                 FROM accounts
                 WHERE account_id = $1 )
                AND devices.device_id = $2
            ON CONFLICT (owner)
            DO UPDATE SET
                public_key = EXCLUDED.public_key,
                signature = EXCLUDED.signature,
                key_id = EXCLUDED.key_id;
            "#,
            aci,
            dev,
            k_id,
            p_key,
            sig
        )
        .execute(&self.pool)
        .await
        {
            Ok(_) => Ok(()),
            Err(err) => {
                error!("The database failed to set a last resort PQ pre key for {account_id}.{device_id}: {err}");
                Err(KeyManagerError::ServiceUnavailable)
            }
        }
    }

    async fn remove_last_resort_key(
        &mut self,
        account_id: AccountId,
        device_id: DeviceId,
    ) -> Result<(), KeyManagerError> {
        let aci = account_id.uuid();
        let dev = *device_id as i64;

        match sqlx::query!(
            r#"
            DELETE FROM pq_pre_keys
            WHERE owner = 
                (SELECT id
                 FROM devices
                 WHERE owner = 
                         (SELECT id
                         FROM accounts
                         WHERE account_id = $1)
                   AND devices.device_id = $2)
            "#,
            aci,
            dev,
        )
        .execute(&self.pool)
        .await
        {
            Ok(res) => {
                if res.rows_affected() < 1 {
                    debug!("The database failed to delete a last resort PQ pre key. It may have been deleted already");
                }
                Ok(())
            }
            Err(err) => {
                error!("Error while removing Last Resort Pq pre key for {account_id}.{device_id}: {err}");
                Err(KeyManagerError::ServiceUnavailable)
            }
        }
    }
}
