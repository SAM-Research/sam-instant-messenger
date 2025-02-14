use derive_more::derive::{Display, Error, From};
use libsignal_core::curve::CurveError;
use libsignal_protocol::SignalProtocolError;
use sam_common::LibError;
use sqlx::{sqlite::SqliteError, Error as SqlxError};

use crate::net::HttpClientError;

#[derive(Debug, Display, Error, From)]
pub enum ClientError {
    #[display("Failed to parse an invalid ServiceId: {_0}")]
    InvalidServiceId(#[error(not(source))] String),
    SignalProtocol(SignalProtocolError),
    Sqlite(SqliteError),
    Sqlx(SqlxError),
    Lib(LibError),
    Curve(CurveError),
    HttpClient(HttpClientError),
}
