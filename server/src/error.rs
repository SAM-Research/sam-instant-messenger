use derive_more::derive::{Display, Error, From};
use sam_common::LibError;

use crate::storage::error::DatabaseError;

#[derive(Debug, Display, Error, From)]
pub enum ServerError {
    #[error(ignore)]
    Custom(String),
    Lib(LibError),
    Database(DatabaseError),
}
