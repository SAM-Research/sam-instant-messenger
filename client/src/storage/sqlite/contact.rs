use sqlx::{Pool, Sqlite};

use crate::storage::ContactStore;

#[derive(Debug)]
pub struct SqliteContactStore {
    _database: Pool<Sqlite>,
}

impl SqliteContactStore {
    pub fn new(database: Pool<Sqlite>) -> Self {
        Self {
            _database: database,
        }
    }
}

impl ContactStore for SqliteContactStore {}
