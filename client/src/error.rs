use derive_more::derive::{Display, Error, From};
use libsignal_core::curve::CurveError;
use libsignal_protocol::SignalProtocolError;
use sam_common::LibError;
use sqlx::{sqlite::SqliteError, Error as SqlxError};
use std::panic::AssertUnwindSafe;

#[derive(Debug, Display, Error, From)]
pub enum ClientError {
    #[display("Failed to parse an invalid ServiceId: {_0}")]
    #[error(ignore)]
    InvalidServiceId(String),
    SignalProtocol(SignalProtocolError),
    Sqlite(SqliteError),
    #[from(ignore)]
    #[display("{}", _0.0)]
    Sqlx(AssertUnwindSafe<SqlxError>),
    Lib(LibError),
    Curve(CurveError),
    NoAccountId,
    NoPassword,
    NoUsername,
}

impl From<SqlxError> for ClientError {
    fn from(value: SqlxError) -> Self {
        match value {
            SqlxError::Database(database_error) => ClientError::Sqlite(*database_error.downcast()),
            SqlxError::RowNotFound => ClientError::Sqlx(AssertUnwindSafe(value)),
            err => panic!("{}", err),
        }
    }
}
