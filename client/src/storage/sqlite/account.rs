use libsignal_protocol::Aci;
use sqlx::{Pool, Sqlite};

use crate::{storage::AccountStore, ClientError};

#[derive(Debug)]
pub struct SqliteAccountStore {
    _database: Pool<Sqlite>,
}

impl SqliteAccountStore {
    pub fn new(database: Pool<Sqlite>) -> Self {
        Self {
            _database: database,
        }
    }
}

impl AccountStore for SqliteAccountStore {
    fn set_aci(&self, _aci: Aci) -> Result<(), ClientError> {
        todo!()
    }

    fn get_aci(&self) -> Result<Aci, ClientError> {
        todo!()
    }

    fn set_password(&self, _password: String) -> Result<(), ClientError> {
        todo!()
    }

    fn get_password(&self) -> Result<String, ClientError> {
        todo!()
    }

    fn set_username(&self, _username: String) -> Result<(), ClientError> {
        todo!()
    }

    fn get_username(&self) -> Result<String, ClientError> {
        todo!()
    }
}
