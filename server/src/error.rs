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
        }
        .into_response()
    }
}
