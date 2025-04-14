use std::collections::HashSet;

use async_trait::async_trait;
use sam_common::{AccountId, DeviceId};
use sqlx::{Pool, Sqlite};

use crate::storage::{
    error::{ContactStoreError, DatabaseError},
    ContactStore,
};

#[derive(Debug)]
pub struct SqliteContactStore {
    database: Pool<Sqlite>,
}

impl SqliteContactStore {
    pub fn new(database: Pool<Sqlite>) -> Self {
        Self { database }
    }
}

#[async_trait(?Send)]
impl ContactStore for SqliteContactStore {
    async fn contains_contact(&self, account_id: AccountId) -> Result<bool, ContactStoreError> {
        let aci_str = account_id.to_string();
        sqlx::query_scalar!(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM Contacts WHERE account_id = ?
            )
            "#,
            aci_str
        )
        .fetch_one(&self.database)
        .await
        .map(|exists| exists == 1)
        .map_err(|err| DatabaseError::Database(format!("{err}")).into())
    }
    async fn get_all_devices(
        &self,
        account_id: AccountId,
    ) -> Result<HashSet<DeviceId>, ContactStoreError> {
        let aci_str = account_id.to_string();
        let ids = sqlx::query!(
            r#"
            SELECT device_id FROM Contacts WHERE account_id = ?
            "#,
            aci_str
        )
        .fetch_all(&self.database)
        .await
        .map_err(|err| DatabaseError::Database(format!("{err}")))?
        .into_iter()
        .map(|rec| (rec.device_id as u32).into());

        Ok(HashSet::from_iter(ids))
    }
    async fn add_device(
        &mut self,
        account_id: AccountId,
        device_id: DeviceId,
    ) -> Result<(), ContactStoreError> {
        let aci_str = account_id.to_string();
        let dev_str = device_id.to_string();
        sqlx::query!(
            r#"
            INSERT INTO Contacts (account_id, device_id)
            SELECT ?, ?
            WHERE NOT EXISTS (
                SELECT 1 FROM Contacts 
                WHERE account_id = ? AND device_id = ?
            )
            "#,
            aci_str,
            dev_str,
            aci_str,
            dev_str,
        )
        .execute(&self.database)
        .await
        .map(|_| ())
        .map_err(|err| DatabaseError::Database(format!("{err}")).into())
    }

    async fn remove_device(
        &mut self,
        account_id: AccountId,
        device_id: DeviceId,
    ) -> Result<(), ContactStoreError> {
        let aci_str = account_id.to_string();
        let dev_str = device_id.to_string();
        sqlx::query!(
            r#"
            DELETE FROM Contacts 
            WHERE account_id=? AND device_id=?
            "#,
            aci_str,
            dev_str,
        )
        .execute(&self.database)
        .await
        .map(|_| ())
        .map_err(|err| DatabaseError::Database(format!("{err}")).into())
    }
}
