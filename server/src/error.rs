use axum::{http::StatusCode, response::IntoResponse};
use derive_more::derive::{Display, Error};
use sam_common::LibError;

pub type Result<T> = std::result::Result<T, ServerError>;

#[derive(Debug, Display, Error)]
pub enum ServerError {
    #[error(ignore)]
    Custom(String),
    Lib(LibError),
    KeyVerification,
    DeviceTokenMalformed,
    DeviceSignatureDecodeError,
    DeviceWrongSignature,
    DeviceLinkTooSlow,
    DeviceProvisionUnAuth,
    AccountIDUnParsable,
    PasswordHashError,
    WrongPassword,
    AuthBasicParseError,
    SocketPollError,
    SocketClientDisconnect,
    SocketDecodeError,
    AccountNotExist,
    AccountExists,
    DeviceNotExist,
    DeviceExists,
}

impl IntoResponse for ServerError {
    fn into_response(self) -> axum::response::Response {
        match self {
            ServerError::Custom(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ServerError::Lib(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ServerError::KeyVerification => StatusCode::INTERNAL_SERVER_ERROR,
            ServerError::DeviceTokenMalformed => StatusCode::INTERNAL_SERVER_ERROR,
            ServerError::DeviceSignatureDecodeError => StatusCode::INTERNAL_SERVER_ERROR,
            ServerError::DeviceWrongSignature => StatusCode::INTERNAL_SERVER_ERROR,
            ServerError::DeviceLinkTooSlow => StatusCode::INTERNAL_SERVER_ERROR,
            ServerError::DeviceProvisionUnAuth => StatusCode::INTERNAL_SERVER_ERROR,
            ServerError::AccountIDUnParsable => StatusCode::INTERNAL_SERVER_ERROR,
            ServerError::PasswordHashError => StatusCode::INTERNAL_SERVER_ERROR,
            ServerError::WrongPassword => StatusCode::INTERNAL_SERVER_ERROR,
            ServerError::AuthBasicParseError => StatusCode::INTERNAL_SERVER_ERROR,
            ServerError::SocketPollError => panic!("SocketPollError cannot happen over HTTP"),
            ServerError::SocketClientDisconnect => {
                panic!("SocketClientDisconnect cannot happen over HTTP")
            }
            ServerError::SocketDecodeError => panic!("SocketDecodeError cannot happen over HTTP"),
            ServerError::AccountNotExist => todo!(),
            ServerError::AccountExists => todo!(),
            ServerError::DeviceNotExist => todo!(),
            ServerError::DeviceExists => todo!(),
        }
        .into_response()
    }
}
