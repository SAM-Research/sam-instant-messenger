use derive_more::derive::{Display, Error, From};
use libsignal_protocol::SignalProtocolError;
use sam_common::LibError;
use sqlx::{sqlite::SqliteError, Error as SqlxError};

pub type Result<T> = std::result::Result<T, ClientError>;

#[derive(Debug, Display, Error, From)]
pub enum ClientError {
    #[error(ignore)]
    Custom(String),
    SignalProtocol(SignalProtocolError),
    Sqlite(SqliteError),
    Lib(LibError),
}

impl From<SqlxError> for ClientError {
    fn from(value: SqlxError) -> Self {
        match value {
            SqlxError::Database(database_error) => ClientError::Sqlite(*database_error.downcast()),
            _ => todo!(),
        }
    }
}
