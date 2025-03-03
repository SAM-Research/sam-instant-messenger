use crate::auth::error::AuthorizationError;
use crate::managers::error::{
    AccountManagerError, DeviceManagerError, KeyManagerError, MessageManagerError,
};
use crate::routes::error::RouterError;
use axum::response::IntoResponse;
use derive_more::derive::{Display, Error};
use derive_more::From;
use log::error;
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
}

impl IntoResponse for ServerError {
    fn into_response(self) -> axum::response::Response {
        error!("ServerError occured: {}", self);
        match self {
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
