use async_trait::async_trait;
use log::{debug, error};
use sam_common::{
    api::{EcPreKey, Key},
    AccountId, DeviceId,
};
use sqlx::{Pool, Postgres};

use crate::managers::{error::KeyManagerError, traits::key_manager::EcPreKeyManager};

#[derive(Debug, Clone)]
pub struct PostgresEcPreKeyManager {
    pool: Pool<Postgres>,
}

impl PostgresEcPreKeyManager {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl EcPreKeyManager for PostgresEcPreKeyManager {
    async fn get_pre_key(
        &self,
        account_id: AccountId,
        device_id: DeviceId,
    ) -> Result<Option<EcPreKey>, KeyManagerError> {
        let aci = account_id.uuid();
        let dev = *device_id as i64;
        match sqlx::query!(
            r#"
            SELECT key_id, public_key
            FROM ec_pre_keys
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
                let key_id = row
                    .key_id
                    .try_into()
                    .map_err(|_| {
                        error!("Device {device_id} belonging to {account_id} has an EC pre key with an invalid ID '{}' ", row.key_id);
                        KeyManagerError::ServiceUnavailable})?;
                Ok(Some(EcPreKey { public_key, key_id }))
            }
            Err(sqlx::Error::RowNotFound) => Ok(None),
            Err(err) => {
                error!("Error while attempting to fetch a pre key: {err}");
                Err(KeyManagerError::ServiceUnavailable)
            }
        }
    }

    async fn get_pre_key_ids(
        &self,
        account_id: AccountId,
        device_id: DeviceId,
    ) -> Result<Option<Vec<u32>>, KeyManagerError> {
        let aci = account_id.uuid();
        let dev = *device_id as i64;
        match sqlx::query!(
            r#"
            SELECT key_id
            FROM ec_pre_keys
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
        .fetch_all(&self.pool)
        .await
        {
            Ok(rows) => Ok(Some(rows.iter().map(|row| row.key_id as u32).collect())),
            Err(err) => {
                error!("Error getting all pre key ids: {err}");
                Err(KeyManagerError::ServiceUnavailable)
            }
        }
    }

    async fn add_pre_key(
        &mut self,
        account_id: AccountId,
        device_id: DeviceId,
        key: EcPreKey,
    ) -> Result<(), KeyManagerError> {
        let aci = account_id.uuid();
        let dev = *device_id as i64;
        let k_id = key.id() as i64;
        let p_key = &*key.public_key;
        match sqlx::query!(
            r#"
            INSERT INTO ec_pre_keys (owner, key_id, public_key)
            SELECT id,
                   $3,
                   $4
            FROM devices
            WHERE owner = 
                (SELECT id
                 FROM accounts
                 WHERE account_id = $1 )
                AND devices.device_id = $2
            "#,
            aci,
            dev,
            k_id,
            p_key
        )
        .execute(&self.pool)
        .await
        {
            Ok(res) => {
                if res.rows_affected() != 1 {
                    error!("The database failed to insert an EC pre key");
                    return Err(KeyManagerError::ServiceUnavailable);
                }
                Ok(())
            }
            Err(err) => {
                error!("Failed to insert EC pre key: {err}");
                Err(KeyManagerError::ServiceUnavailable)
            }
        }
    }

    async fn remove_pre_key(
        &mut self,
        account_id: AccountId,
        device_id: DeviceId,
        id: u32,
    ) -> Result<(), KeyManagerError> {
        let aci = account_id.uuid();
        let dev = *device_id as i64;

        match sqlx::query!(
            r#"
            DELETE FROM ec_pre_keys
            WHERE owner = 
                (SELECT id
                 FROM devices
                 WHERE owner = 
                         (SELECT id
                         FROM accounts
                         WHERE account_id = $1)
                   AND devices.device_id = $2)
            AND key_id = $3
            "#,
            aci,
            dev,
            id as i64,
        )
        .execute(&self.pool)
        .await
        {
            Ok(res) => {
                if res.rows_affected() < 1 {
                    debug!("The database did not delete an EC pre key. It may have already been deleted");
                }
                Ok(())
            }
            Err(err) => {
                error!("Error while removing EC pre key with ID {id} for {account_id}.{device_id}: {err}");
                Err(KeyManagerError::ServiceUnavailable)
            }
        }
    }
}
