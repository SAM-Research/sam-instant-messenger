use async_trait::async_trait;
use libsignal_core::Aci;

use crate::{storage::AccountStore, ClientError};

#[derive(Debug, Default)]
pub struct InMemoryAccountStore {}

#[async_trait(?Send)]
impl AccountStore for InMemoryAccountStore {
    async fn set_aci(&self, _aci: Aci) -> Result<(), ClientError> {
        todo!()
    }
    async fn get_aci(&self) -> Result<Aci, ClientError> {
        todo!()
    }
    async fn set_password(&self, _password: String) -> Result<(), ClientError> {
        todo!()
    }
    async fn get_password(&self) -> Result<String, ClientError> {
        todo!()
    }
    async fn set_username(&self, _username: String) -> Result<(), ClientError> {
        todo!()
    }
    async fn get_username(&self) -> Result<String, ClientError> {
        todo!()
    }
}
