use uuid::Uuid;

use crate::{state::entities::account::Account, ServerError};

#[async_trait::async_trait]
pub trait AccountManager: Send {
    async fn get_account(&self, id: &Uuid) -> Result<Account, ServerError>;

    async fn add_account(&mut self, account: &Account) -> Result<(), ServerError>;
    async fn update_account(&mut self, id: &Uuid, account: Account) -> Result<(), ServerError>;
    async fn remove_account(&mut self, account_id: Uuid) -> Result<(), ServerError>;
}
