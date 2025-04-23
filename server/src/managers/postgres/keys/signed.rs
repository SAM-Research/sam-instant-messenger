use async_trait::async_trait;
use libsignal_protocol::IdentityKey;
use log::{debug, error};
use sam_common::{
    api::{Key as _, SignedEcPreKey},
    AccountId, DeviceId,
};
use sqlx::{Pool, Postgres};

use crate::{
    auth::keys::verify_key,
    managers::{error::KeyManagerError, traits::key_manager::SignedPreKeyManager},
};

#[derive(Debug, Clone)]
pub struct PostgresSignedPreKeyManager {
    pool: Pool<Postgres>,
}

impl PostgresSignedPreKeyManager {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SignedPreKeyManager for PostgresSignedPreKeyManager {
    async fn get_signed_pre_key(
        &self,
        account_id: AccountId,
        device_id: DeviceId,
    ) -> Result<SignedEcPreKey, KeyManagerError> {
        let aci = account_id.uuid();
        let dev = *device_id as i64;
        match sqlx::query!(
            r#"
            SELECT key_id, public_key, signature
            FROM signed_pre_keys
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
                        error!("Device {device_id} belonging to {account_id} has a Signed EC Pre Key with an invalid ID '{}' ", row.key_id);
                        KeyManagerError::ServiceUnavailable})?;
                Ok(SignedEcPreKey {
                    public_key,
                    key_id,
                    signature,
                })
            }
            Err(sqlx::Error::RowNotFound) => Err(KeyManagerError::KeyDoesNotExist),
            Err(err) => {
                error!("Error while attempting to fetch a Signed EC Pre Key for {account_id}.{device_id}: {err}");
                Err(KeyManagerError::ServiceUnavailable)
            }
        }
    }

    async fn set_signed_pre_key(
        &mut self,
        account_id: AccountId,
        device_id: DeviceId,
        identity: &IdentityKey,
        key: SignedEcPreKey,
    ) -> Result<(), KeyManagerError> {
        let aci = account_id.uuid();
        let dev = *device_id as i64;
        let k_id = key.id() as i64;

        verify_key(identity, &key)?;

        let p_key = &*key.public_key;
        let sig = &*key.signature;
        match sqlx::query!(
            r#"
            INSERT INTO signed_pre_keys (owner, key_id, public_key, signature)
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
                error!("Failed to insert Signed EC Pre Key for {account_id}.{device_id}: {err}");
                Err(KeyManagerError::ServiceUnavailable)
            }
        }
    }

    async fn remove_signed_pre_key(
        &mut self,
        account_id: AccountId,
        device_id: DeviceId,
    ) -> Result<(), KeyManagerError> {
        let aci = account_id.uuid();
        let dev = *device_id as i64;

        match sqlx::query!(
            r#"
            DELETE FROM signed_pre_keys
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
                if res.rows_affected() != 1 {
                    debug!("The database failed to delete a Signed EC Pre Key for {account_id}.{device_id}. It may have already been deleted");
                }
                Ok(())
            }
            Err(err) => {
                error!(
                    "Error while removing Signed EC Pre Key for {account_id}.{device_id}: {err}"
                );
                Err(KeyManagerError::ServiceUnavailable)
            }
        }
    }
}
