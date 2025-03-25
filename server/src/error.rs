use crate::auth::error::AuthorizationError;
use crate::managers::error::{
    AccountManagerError, DeviceManagerError, KeyManagerError, MessageManagerError,
};
use crate::routes::error::RouterError;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use derive_more::derive::{Display, Error};
use derive_more::From;
use log::error;
use rustls::server::VerifierBuilderError;
use sam_common::LibError;

pub type Result<T> = std::result::Result<T, ServerError>;

#[derive(Debug, Display, Error, From)]
pub enum ServerError {
    #[error(ignore)]
    Lib(LibError),
    AccountManager(AccountManagerError),
    DeviceManager(DeviceManagerError),
    KeyManager(KeyManagerError),
    MessageManager(MessageManagerError),
    Authorization(AuthorizationError),
    Router(RouterError),
    EnvelopeMalformed,
}

#[derive(Debug, Display, Error, From)]
pub enum TLSError {
    LoadError(std::io::Error),
    VerifierError(VerifierBuilderError),
    RustlsError(rustls::Error),
    PrivateKeyWasNone,
}

#[derive(Debug, Display, Error, From)]
pub enum TLSConfigError {
    LoadError(std::io::Error),
    DecodeError(serde_json::Error),
}

#[derive(Debug, Display, Error, From)]
pub enum CLIError {
    TLSConfigError(TLSConfigError),
    TLSError(TLSError),
    AddressParseError,
    FailedToStartServer,
    #[error(ignore)]
    ArgumentError(String),
}

impl IntoResponse for ServerError {
    fn into_response(self) -> axum::response::Response {
        error!("ServerError occured: {}", self);
        match self {
            ServerError::EnvelopeMalformed => {
                (StatusCode::BAD_REQUEST, "Message is malformed".to_string()).into_response()
            }
            ServerError::Lib(error) => error.into_response(),
            ServerError::MessageManager(error) => error.into_response(),
            ServerError::AccountManager(error) => error.into_response(),
            ServerError::KeyManager(error) => error.into_response(),
            ServerError::DeviceManager(error) => error.into_response(),
            ServerError::Authorization(error) => error.into_response(),
            ServerError::Router(error) => error.into_response(),
        }
    }
}
