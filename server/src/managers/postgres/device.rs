use async_trait::async_trait;
use log::error;
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
}

#[async_trait]
impl DeviceManager for PostgresDeviceManager {
    async fn get_device(
        &self,
        account_id: AccountId,
        device_id: DeviceId,
    ) -> Result<Device, DeviceManagerError> {
        let aci = account_id.uuid();
        sqlx::query!(
            r#"
            SELECT device_id,
                   name,
                   hash,
                   salt,
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
        .map(|row| {
            let password = Password::builder().hash(row.hash).salt(row.salt).build();
            Device::builder()
                .name(row.name)
                .id((row.device_id as u32).into())
                .registration_id((row.registration_id as u32).into())
                .password(password)
                .build()
        })
        .map_err(|err| match err {
            //TODO: WE DON'T KNOW WHY WE COULD NOT FIND A ROW
            sqlx::Error::RowNotFound => DeviceManagerError::DeviceDoesNotExist,
            _ => todo!(),
        })
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
                    return Err(DeviceManagerError::NoDevicesFound);
                }
                Ok(rows
                    .iter()
                    .map(|record| (record.device_id as u32).into())
                    .collect())
            }
            Err(err) => {
                error!("Error while getting all device IDs for {account_id}: {err}");
                Err(DeviceManagerError::ServiceUnavailable)
            }
        }
    }

    async fn next_device_id(&self, account_id: AccountId) -> Result<DeviceId, DeviceManagerError> {
        let aci = account_id.uuid();
        sqlx::query!(
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
        .map(|row| (row.max.map(|id| id + 1).unwrap_or_default() as u32).into())
        .map_err(|err| match err {
            sqlx::Error::RowNotFound => DeviceManagerError::DeviceDoesNotExist,
            _ => todo!(),
        })
    }

    async fn link_secret(&self) -> Result<String, DeviceManagerError> {
        sqlx::query!(
            r#"
            SELECT link_secret
            FROM device_link_info
            "#,
        )
        .fetch_one(&self.pool)
        .await
        .map(|row| row.link_secret)
        .map_err(|err| match err {
            sqlx::Error::RowNotFound => DeviceManagerError::ServiceUnavailable,
            _ => todo!(),
        })
    }

    async fn provision_expire_seconds(&self) -> Result<u32, DeviceManagerError> {
        sqlx::query!(
            r#"
            SELECT provision_expire_seconds 
            FROM device_link_info
            "#,
        )
        .fetch_one(&self.pool)
        .await
        .map(|row| (row.provision_expire_seconds as u32).into())
        .map_err(|err| match err {
            sqlx::Error::RowNotFound => DeviceManagerError::ServiceUnavailable,
            _ => todo!(),
        })
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
            INSERT INTO devices (owner, device_id, registration_id, name, hash, salt) 
            SELECT id,
                   $2,
                   $3,
                   $4,
                   $5,
                   $6
            FROM accounts
            WHERE account_id = $1
            "#,
            aci,
            (*device.id()) as i64,
            (*device.registration_id()) as i64,
            device.name(),
            pwd.hash(),
            pwd.salt().to_string()
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
                error!("{err}");
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
