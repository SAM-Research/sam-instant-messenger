use derive_more::derive::{Display, Error, From};
use libsignal_protocol::SignalProtocolError;
use sam_common::LibError;
use sqlx::{sqlite::SqliteError, Error as SqlxError};

#[derive(Debug, Display, Error, From)]
pub enum ClientError {
    #[display("Failed to parse an invalid ServiceId: {_0}")]
    #[error(ignore)]
    InvalidServiceId(String),
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
