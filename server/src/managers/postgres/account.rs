use libsignal_protocol::IdentityKey;
use log::error;
use sam_common::AccountId;
use sqlx::{postgres::PgDatabaseError, types::Uuid, Pool, Postgres};

use crate::managers::{
    entities::Account, error::AccountManagerError, traits::account_manager::AccountManager,
};

#[derive(Debug, Clone)]
pub struct PostgresAccountManager {
    pool: Pool<Postgres>,
}

impl PostgresAccountManager {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl AccountManager for PostgresAccountManager {
    async fn get_account(&self, id: AccountId) -> Result<Account, AccountManagerError> {
        let uuid = id.uuid();
        match sqlx::query!(
            r#"
            SELECT * FROM accounts
            WHERE account_id = $1
            "#,
            uuid
        )
        .fetch_one(&self.pool)
        .await
        {
            Ok(row) => {
                let identity = IdentityKey::decode(row.identity_key.as_slice())
                    .inspect_err(|err| error!("Error loading account UUID from database: {err}"))
                    .map_err(|_| AccountManagerError::MalformedData)?;
                Ok(Account::builder()
                    .id(row.account_id.into())
                    .identity(identity)
                    .username(row.username)
                    .build())
            }
            Err(sqlx::Error::RowNotFound) => Err(AccountManagerError::AccountDoesNotExist),
            Err(err) => {
                error!("Could not fetch account from database: {err}");
                Err(AccountManagerError::ServiceUnavailable)
            }
        }
    }

    async fn get_account_id_from_username(
        &self,
        username: String,
    ) -> Result<AccountId, AccountManagerError> {
        match sqlx::query!(
            r#"
            SELECT (account_id) FROM accounts
            WHERE username = $1
            "#,
            username
        )
        .fetch_one(&self.pool)
        .await
        {
            Ok(row) => Ok(row.account_id.into()),
            Err(sqlx::Error::RowNotFound) => Err(AccountManagerError::AccountDoesNotExist),
            Err(err) => {
                error!("Could not fetch username from database: {err}");
                Err(AccountManagerError::ServiceUnavailable)
            }
        }
    }

    async fn add_account(&mut self, account: &Account) -> Result<(), AccountManagerError> {
        let id: Uuid = account.id().into();
        let username = account.username();
        let identity = account.identity().serialize().to_vec();
        match sqlx::query!(
            r#"
            INSERT INTO accounts (account_id, username, identity_key)
            VALUES ($1, $2, $3)
            "#,
            id,
            username,
            &identity
        )
        .execute(&self.pool)
        .await
        {
            Ok(_) => Ok(()),
            Err(sqlx::Error::Database(err)) => {
                let err: PgDatabaseError = *err.downcast();
                if let Some(constraint) = err.constraint() {
                    if constraint == "accounts_account_id_key" {
                        return Err(AccountManagerError::AccountAlreadyExists);
                    } else if constraint == "accounts_username_key" {
                        return Err(AccountManagerError::UsernameAlreadyExists);
                    }
                }

                error!("Could not save account in database: {err}");
                return Err(AccountManagerError::ServiceUnavailable);
            }
            Err(err) => {
                error!("Could not save account in database: {err}");
                Err(AccountManagerError::ServiceUnavailable)
            }
        }
    }

    async fn remove_account(&mut self, account_id: AccountId) -> Result<(), AccountManagerError> {
        let account_id: Uuid = account_id.into();
        match sqlx::query!(
            r#"
            DELETE FROM accounts 
            WHERE account_id = $1
            RETURNING (account_id)
            "#,
            account_id,
        )
        .fetch_one(&self.pool)
        .await
        {
            Ok(_) => Ok(()),
            Err(sqlx::Error::RowNotFound) => Err(AccountManagerError::AccountDoesNotExist),
            Err(err) => {
                error!("Could not remove account from database: {err}");
                Err(AccountManagerError::ServiceUnavailable)
            }
        }
    }
}

#[cfg(test)]
mod test {
    use libsignal_protocol::IdentityKeyPair;
    use rand::rngs::OsRng;
    use rstest::fixture;
    use sam_common::AccountId;
    use sqlx::{types::Uuid, Pool, Postgres};

    use crate::managers::{
        entities::Account, error::AccountManagerError, traits::account_manager::AccountManager,
    };

    use super::PostgresAccountManager;

    #[fixture]
    fn connection_str() -> &'static str {
        "postgres://test:test@127.0.0.1:5432/sam_test_db"
    }

    #[fixture]
    async fn accounts(connection_str: &str) -> PostgresAccountManager {
        let pool = Pool::<Postgres>::connect(connection_str)
            .await
            .expect("Can connect to postgres");

        PostgresAccountManager::new(pool)
    }

    #[tokio::test]
    async fn postgres_account_manager() {
        let _ = env_logger::try_init();
        let mut manager = accounts(connection_str()).await;
        let username = Uuid::new_v4();
        let account = Account::builder()
            .id(AccountId::generate())
            .identity(
                IdentityKeyPair::generate(&mut OsRng)
                    .identity_key()
                    .to_owned(),
            )
            .username(username.to_string())
            .build();
        assert!(manager.add_account(&account).await.is_ok());

        assert_eq!(
            account,
            manager
                .get_account(account.id())
                .await
                .expect("Can get account that was just inserted")
        );

        assert_eq!(
            account.id(),
            manager
                .get_account_id_from_username(account.username().to_owned())
                .await
                .expect("Can get id of account that was just inserted")
        );

        assert!(manager.remove_account(account.id()).await.is_ok());
    }

    #[tokio::test]
    async fn postgres_account_manager_get_account_does_not_exist() {
        let manager = accounts(connection_str()).await;

        let account_id = AccountId::generate();

        assert!(matches!(
            manager.get_account(account_id).await,
            Err(AccountManagerError::AccountDoesNotExist)
        ));
    }

    #[tokio::test]
    async fn postgres_account_manager_remove_account_does_not_exist() {
        let mut manager = accounts(connection_str()).await;

        let account_id = AccountId::generate();

        assert!(matches!(
            manager.remove_account(account_id).await,
            Err(AccountManagerError::AccountDoesNotExist)
        ));
    }

    #[tokio::test]
    async fn postgres_account_manager_removed_account_cannot_be_retrieved() {
        let mut manager = accounts(connection_str()).await;
        let username = Uuid::new_v4();
        let account = Account::builder()
            .id(AccountId::generate())
            .identity(
                IdentityKeyPair::generate(&mut OsRng)
                    .identity_key()
                    .to_owned(),
            )
            .username(username.to_string())
            .build();
        assert!(manager.add_account(&account).await.is_ok());
        assert!(manager.remove_account(account.id()).await.is_ok());
        assert!(matches!(
            manager.get_account(account.id()).await,
            Err(AccountManagerError::AccountDoesNotExist)
        ));
    }

    #[tokio::test]
    async fn postgres_account_manager_get_by_username_account_does_not_exist() {
        let manager = accounts(connection_str()).await;

        assert!(matches!(
            manager
                .get_account_id_from_username("DoesNotExist".to_owned())
                .await,
            Err(AccountManagerError::AccountDoesNotExist)
        ));
    }

    #[tokio::test]
    async fn postgres_account_manager_cannot_insert_duplicate_ids() {
        let mut manager = accounts(connection_str()).await;
        let id = AccountId::generate();
        let username = Uuid::new_v4();
        let account1 = Account::builder()
            .id(id)
            .identity(
                IdentityKeyPair::generate(&mut OsRng)
                    .identity_key()
                    .to_owned(),
            )
            .username(username.to_string())
            .build();
        assert!(manager.add_account(&account1).await.is_ok());

        let new_username = Uuid::new_v4();
        let account2 = Account::builder()
            .id(id)
            .identity(
                IdentityKeyPair::generate(&mut OsRng)
                    .identity_key()
                    .to_owned(),
            )
            .username(new_username.to_string())
            .build();

        assert!(matches!(
            manager.add_account(&account2).await,
            Err(AccountManagerError::AccountAlreadyExists)
        ));
    }

    #[tokio::test]
    async fn postgres_account_manager_cannot_insert_duplicate_usernames() {
        let mut manager = accounts(connection_str()).await;
        let username = Uuid::new_v4();
        let account1 = Account::builder()
            .id(AccountId::generate())
            .identity(
                IdentityKeyPair::generate(&mut OsRng)
                    .identity_key()
                    .to_owned(),
            )
            .username(username.to_string())
            .build();
        assert!(manager.add_account(&account1).await.is_ok());

        let account2 = Account::builder()
            .id(AccountId::generate())
            .identity(
                IdentityKeyPair::generate(&mut OsRng)
                    .identity_key()
                    .to_owned(),
            )
            .username(username.to_string())
            .build();

        assert!(matches!(
            manager.add_account(&account2).await,
            Err(AccountManagerError::UsernameAlreadyExists)
        ));
    }
}
