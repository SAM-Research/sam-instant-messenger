use async_trait::async_trait;
use log::{debug, error};
use sam_common::{AccountId, DeviceId};
use sqlx::{postgres::PgDatabaseError, Pool, Postgres};

use crate::{
    auth::password::Password,
    managers::{
        entities::Device, error::DeviceManagerError, traits::device_manager::DeviceManager,
    },
};

#[derive(Debug, Clone)]
pub struct PostgresDeviceManager {
    pool: Pool<Postgres>,
}

impl PostgresDeviceManager {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }

    pub async fn create(
        pool: Pool<Postgres>,
        link_secret: &str,
        provision_expire_seconds: u32,
    ) -> Result<Self, DeviceManagerError> {
        match sqlx::query!(
            r#"
            INSERT INTO device_link_info (id, link_secret, provision_expire_seconds)
            VALUES (1, $1, $2)
            ON CONFLICT (id)
            DO UPDATE SET
                link_secret = excluded.link_secret,
                provision_expire_seconds = excluded.provision_expire_seconds
            "#,
            link_secret,
            provision_expire_seconds as i64
        )
        .execute(&pool)
        .await
        {
            Ok(res) => {
                if res.rows_affected() != 1 {
                    error!("The database did not insert the link secret");
                    return Err(DeviceManagerError::ServiceUnavailable);
                }
                let manager = Self::new(pool);
                Ok(manager)
            }
            Err(err) => {
                error!("Could not store link secret in database: {err}");
                Err(DeviceManagerError::ServiceUnavailable)
            }
        }
    }

    pub async fn set_link_secret(&self, link_secret: &str) -> Result<(), DeviceManagerError> {
        match sqlx::query!(
            r#"
            UPDATE device_link_info
            SET link_secret = $1
            WHERE id = 1
            "#,
            link_secret
        )
        .execute(&self.pool)
        .await
        {
            Ok(res) => {
                if res.rows_affected() != 1 {
                    error!("The database did not insert the link secret");
                    return Err(DeviceManagerError::ServiceUnavailable);
                }
                Ok(())
            }
            Err(err) => {
                error!("Could not store link secret in database: {err}");
                Err(DeviceManagerError::ServiceUnavailable)
            }
        }
    }

    pub async fn set_provision_expire_seconds(
        &self,
        provision_expire_seconds: u32,
    ) -> Result<(), DeviceManagerError> {
        match sqlx::query!(
            r#"
            UPDATE device_link_info
            SET provision_expire_seconds = $1
            WHERE id = 1
            "#,
            provision_expire_seconds as i64
        )
        .execute(&self.pool)
        .await
        {
            Ok(res) => {
                if res.rows_affected() != 1 {
                    error!("The database did not insert the provision expire time");
                    return Err(DeviceManagerError::ServiceUnavailable);
                }
                Ok(())
            }
            Err(err) => {
                error!("Could not store link secret in database: {err}");
                Err(DeviceManagerError::ServiceUnavailable)
            }
        }
    }
}

#[async_trait]
impl DeviceManager for PostgresDeviceManager {
    async fn get_device(
        &self,
        account_id: AccountId,
        device_id: DeviceId,
    ) -> Result<Device, DeviceManagerError> {
        let aci = account_id.uuid();
        match sqlx::query!(
            r#"
            SELECT device_id,
                   name,
                   hash,
                   registration_id
            FROM devices
            WHERE owner =
                    (SELECT id
                     FROM accounts
                     WHERE account_id = $1)
            AND device_id = $2
            "#,
            aci,
            *device_id as i64
        )
        .fetch_one(&self.pool)
        .await
        {
            Ok(row) => {
                let password = Password::builder().hash(row.hash).build();
                let device_id = u32::try_from(row.device_id)
                    .inspect_err(|_| {
                        error!(
                            "Error parsing device ID from database - ID was {}",
                            row.device_id
                        )
                    })
                    .map_err(|_| DeviceManagerError::ServiceUnavailable)?;

                let registration_id = u32::try_from(row.registration_id)
                    .inspect_err(|_| {
                        error!(
                            "Error parsing registration ID from database - ID was {}",
                            row.registration_id
                        )
                    })
                    .map_err(|_| DeviceManagerError::ServiceUnavailable)?;

                Ok(Device::builder()
                    .name(row.name)
                    .id(device_id.into())
                    .registration_id(registration_id.into())
                    .password(password)
                    .build())
            }
            Err(sqlx::Error::RowNotFound) => Err(DeviceManagerError::DeviceDoesNotExist),
            Err(err) => {
                error!("Could not device with id {device_id} for account {account_id}: {err}");
                Err(DeviceManagerError::ServiceUnavailable)
            }
        }
    }

    async fn get_devices(
        &self,
        account_id: AccountId,
    ) -> Result<Vec<DeviceId>, DeviceManagerError> {
        let aci = account_id.uuid();
        match sqlx::query!(
            r#"
            SELECT device_id
            FROM devices
            WHERE owner =
                    (SELECT id
                     FROM accounts
                     WHERE account_id = $1)
            "#,
            aci,
        )
        .fetch_all(&self.pool)
        .await
        {
            Ok(rows) => {
                if rows.is_empty() {
                    debug!("Tried to get all devices for {account_id}, but there were none");
                    return Err(DeviceManagerError::NoDevicesFound);
                }
                let mut results = Vec::new();
                for result in rows.iter().map(|record| {
                    u32::try_from(record.device_id)
                        .inspect_err(|_| {
                            error!(
                                "Error parsing device ID from database - ID was {}",
                                record.device_id
                            )
                        })
                        .map_err(|_| DeviceManagerError::ServiceUnavailable)
                }) {
                    results.push(result?.into());
                }

                Ok(results)
            }
            Err(err) => {
                error!("Error while getting all device IDs for {account_id}: {err}");
                Err(DeviceManagerError::ServiceUnavailable)
            }
        }
    }

    async fn next_device_id(&self, account_id: AccountId) -> Result<DeviceId, DeviceManagerError> {
        let aci = account_id.uuid();
        match sqlx::query!(
            r#"
            SELECT MAX(device_id)
            FROM devices
            WHERE owner =
                    (SELECT id
                     FROM accounts
                     WHERE account_id = $1)
            "#,
            aci,
        )
        .fetch_one(&self.pool)
        .await
        {
            Ok(row) => {
                let previous_max = row.max.map(u32::try_from);
                match previous_max {
                    None => {
                        error!("next Device ID query returned a record but it contained NULL");
                        Ok(1.into())
                    }

                    Some(Err(err)) => {
                        error!("Could not generate next ID: {err}");
                        Err(DeviceManagerError::ServiceUnavailable)
                    }

                    Some(Ok(id)) => Ok((id + 1).into()),
                }
            }
            Err(sqlx::Error::RowNotFound) => Ok(1.into()),
            Err(err) => {
                error!(
                    "Could not get previous IDs while trying to generate the next device ID: {err}"
                );
                Err(DeviceManagerError::ServiceUnavailable)
            }
        }
    }

    async fn link_secret(&self) -> Result<String, DeviceManagerError> {
        match sqlx::query!(
            r#"
            SELECT link_secret
            FROM device_link_info
            "#,
        )
        .fetch_one(&self.pool)
        .await
        {
            Ok(row) => Ok(row.link_secret),
            Err(err) => {
                error!("Could not fetch link secret from database: {err}");
                Err(DeviceManagerError::ServiceUnavailable)
            }
        }
    }

    async fn provision_expire_seconds(&self) -> Result<u32, DeviceManagerError> {
        match sqlx::query!(
            r#"
            SELECT provision_expire_seconds 
            FROM device_link_info
            "#,
        )
        .fetch_one(&self.pool)
        .await {
            Ok(row) => {
                u32::try_from(row.provision_expire_seconds).map_err(|_| {
                    error!("provision_expire_seconds is set too high in the database and cannot be converted to u32");
                    DeviceManagerError::ServiceUnavailable
                })
            }
            Err(err) => {
                error!("Could not fetch provision_expire_seconds from database: {err}");
                Err(DeviceManagerError::ServiceUnavailable)
            }

        }
    }

    async fn add_device(
        &mut self,
        account_id: AccountId,
        device: &Device,
    ) -> Result<(), DeviceManagerError> {
        let aci = account_id.uuid();
        let pwd = device.password();
        match sqlx::query!(
            r#"
            INSERT INTO devices (owner, device_id, registration_id, name, hash) 
            SELECT id,
                   $2,
                   $3,
                   $4,
                   $5
            FROM accounts
            WHERE account_id = $1
            "#,
            aci,
            (*device.id()) as i64,
            (*device.registration_id()) as i64,
            device.name(),
            pwd.hash(),
        )
        .execute(&self.pool)
        .await
        {
            Ok(res) => {
                if res.rows_affected() != 1 {
                    error!("The database did not insert the device");
                    return Err(DeviceManagerError::ServiceUnavailable);
                }
                Ok(())
            }
            Err(sqlx::Error::Database(err)) => {
                let err: PgDatabaseError = *err.downcast();
                if let Some(constraint) = err.constraint() {
                    if constraint == "devices_owner_device_id_key" {
                        return Err(DeviceManagerError::DeviceAlreadyExists);
                    }
                }
                error!("Unexpected database error while trying to insert a device: {err}");
                Err(DeviceManagerError::ServiceUnavailable)
            }
            Err(err) => {
                error!("Error while adding device to database: {err}");
                Err(DeviceManagerError::ServiceUnavailable)
            }
        }
    }

    async fn remove_device(
        &mut self,
        account_id: AccountId,
        device_id: DeviceId,
    ) -> Result<(), DeviceManagerError> {
        let aci = account_id.uuid();
        let dev = *device_id as i64;
        match sqlx::query!(
            r#"
            DELETE FROM devices 
            WHERE owner =
                    (SELECT id
                     FROM accounts
                     WHERE account_id = $1)
            AND device_id = $2
            RETURNING (device_id)
            "#,
            aci,
            dev
        )
        .fetch_one(&self.pool)
        .await
        {
            Ok(_) => Ok(()),
            Err(sqlx::Error::RowNotFound) => Err(DeviceManagerError::DeviceDoesNotExist),
            Err(err) => {
                error!("Could not remove device from database: {err}");
                Err(DeviceManagerError::ServiceUnavailable)
            }
        }
    }
}

#[cfg(test)]
mod test {
    use rand::rngs::OsRng;
    use sam_common::address::RegistrationId;
    use sqlx::types::Uuid;

    use crate::{
        auth::password::Password,
        managers::{
            entities::{Account, Device},
            error::DeviceManagerError,
            postgres::test_utils::{accounts, connection_str, devices},
            traits::{account_manager::AccountManager as _, device_manager::DeviceManager},
        },
    };

    #[tokio::test]
    #[ignore = "requires a postgres test database"]
    async fn postgres_device_manager() {
        let conn_str = connection_str();
        let mut acc_manager = accounts(conn_str).await;
        let username = Uuid::new_v4();
        let account = Account::random();

        assert!(acc_manager.add_account(&account).await.is_ok());

        let mut dev_manager = devices(conn_str).await;
        let device_id = 22.into();
        let device = Device::builder()
            .id(device_id)
            .name(username.to_string())
            .password(Password::generate(username.to_string()).expect("can create password"))
            .registration_id(RegistrationId::generate(&mut OsRng))
            .build();

        assert!(dev_manager.add_device(account.id(), &device).await.is_ok());

        assert!(dev_manager
            .get_devices(account.id())
            .await
            .is_ok_and(|devices| devices == vec![device_id]));

        assert!(dev_manager
            .get_device(account.id(), device_id)
            .await
            .is_ok_and(|dev| dev == device));

        assert!(dev_manager
            .remove_device(account.id(), device_id)
            .await
            .is_ok());

        assert!(dev_manager
            .get_devices(account.id())
            .await
            .is_err_and(|err| matches!(err, DeviceManagerError::NoDevicesFound)));

        assert!(dev_manager
            .get_device(account.id(), device_id)
            .await
            .is_err_and(|err| matches!(err, DeviceManagerError::DeviceDoesNotExist)))
    }

    #[tokio::test]
    #[ignore = "requires a postgres test database"]
    async fn postgres_device_manager_cannot_get_device_that_does_not_exist() {
        let conn_str = connection_str();
        let account = Account::random();

        let dev_manager = devices(conn_str).await;
        let device_id = 22.into();

        assert!(dev_manager
            .get_device(account.id(), device_id)
            .await
            .is_err_and(|err| matches!(err, DeviceManagerError::DeviceDoesNotExist)))
    }

    #[tokio::test]
    #[ignore = "requires a postgres test database"]
    async fn postgres_device_manager_cannot_remove_device_that_does_not_exist() {
        let conn_str = connection_str();
        let mut dev_manager = devices(conn_str).await;
        let account = Account::random();
        let device_id = 22.into();

        assert!(dev_manager
            .remove_device(account.id(), device_id)
            .await
            .is_err_and(|err| matches!(err, DeviceManagerError::DeviceDoesNotExist)))
    }

    #[tokio::test]
    #[ignore = "requires a postgres test database"]
    async fn postgres_device_manager_removed_device_cannot_be_retrieved() {
        let conn_str = connection_str();
        let mut acc_manager = accounts(conn_str).await;
        let account = Account::random();

        assert!(acc_manager.add_account(&account).await.is_ok());

        let mut dev_manager = devices(conn_str).await;
        let device_id = 22.into();
        let device_name = Uuid::new_v4();
        let device = Device::builder()
            .id(device_id)
            .name(device_name.to_string())
            .password(Password::generate(device_name.to_string()).expect("can create password"))
            .registration_id(RegistrationId::generate(&mut OsRng))
            .build();

        assert!(dev_manager.add_device(account.id(), &device).await.is_ok());

        assert!(dev_manager
            .get_device(account.id(), device_id)
            .await
            .is_ok_and(|dev| dev == device));

        assert!(dev_manager
            .remove_device(account.id(), device_id)
            .await
            .is_ok());

        assert!(dev_manager
            .get_device(account.id(), device_id)
            .await
            .is_err_and(|err| matches!(err, DeviceManagerError::DeviceDoesNotExist)))
    }

    #[tokio::test]
    #[ignore = "requires a postgres test database"]
    async fn postgres_device_manager_get_devices_returns_err_if_no_devices() {
        let conn_str = connection_str();
        let mut acc_manager = accounts(conn_str).await;
        let account = Account::random();

        assert!(acc_manager.add_account(&account).await.is_ok());

        let dev_manager = devices(conn_str).await;

        assert!(dev_manager
            .get_devices(account.id())
            .await
            .is_err_and(|err| matches!(err, DeviceManagerError::NoDevicesFound)))
    }

    #[tokio::test]
    #[ignore = "requires a postgres test database"]
    async fn postgres_device_manager_cannot_insert_duplicate_ids() {
        let conn_str = connection_str();
        let mut acc_manager = accounts(conn_str).await;
        let username = Uuid::new_v4();
        let account = Account::random();

        assert!(acc_manager.add_account(&account).await.is_ok());

        let mut dev_manager = devices(conn_str).await;
        let device_id = 22.into();
        let device = Device::builder()
            .id(device_id)
            .name(username.to_string())
            .password(Password::generate(username.to_string()).expect("can create password"))
            .registration_id(RegistrationId::generate(&mut OsRng))
            .build();

        assert!(dev_manager.add_device(account.id(), &device).await.is_ok());

        assert!(dev_manager
            .add_device(account.id(), &device)
            .await
            .is_err_and(|err| matches!(err, DeviceManagerError::DeviceAlreadyExists)))
    }

    #[tokio::test]
    #[ignore = "requires a postgres test database"]
    async fn postgres_device_manager_next_device_id_is_incremented() {
        let conn_str = connection_str();
        let mut acc_manager = accounts(conn_str).await;
        let username = Uuid::new_v4();
        let account = Account::random();

        assert!(acc_manager.add_account(&account).await.is_ok());

        let mut dev_manager = devices(conn_str).await;
        let device_id = 22.into();
        let device = Device::builder()
            .id(device_id)
            .name(username.to_string())
            .password(Password::generate(username.to_string()).expect("can create password"))
            .registration_id(RegistrationId::generate(&mut OsRng))
            .build();

        assert!(dev_manager.add_device(account.id(), &device).await.is_ok());

        assert!(
            *dev_manager
                .next_device_id(account.id())
                .await
                .expect("Can get next id")
                == *device_id + 1
        )
    }
}
