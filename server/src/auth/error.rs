use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use derive_more::{Display, Error, From};
use log::error;

#[derive(Debug, Display, Error, From)]
pub enum AuthorizationError {
    WrongPassword,
    AuthBasicParseError,
    PasswordHashError,
    KeyVerification,
    DeviceTokenMalformed,
    DeviceSignatureDecodeError,
    DeviceWrongSignature,
    DeviceLinkTooSlow,
}

impl IntoResponse for AuthorizationError {
    fn into_response(self) -> Response {
        error!("AuthorizationError occured {}", self);

        match self {
            AuthorizationError::WrongPassword => {
                (StatusCode::UNAUTHORIZED, "Wrong password.".to_string())
            }
            AuthorizationError::AuthBasicParseError => (
                StatusCode::UNAUTHORIZED,
                "Failed to authorize client.".to_string(),
            ),
            AuthorizationError::PasswordHashError => (
                StatusCode::UNAUTHORIZED,
                "Failed to hash password".to_string(),
            ),
            AuthorizationError::KeyVerification => {
                (StatusCode::UNAUTHORIZED, "Failed to verify key".to_string())
            }
            AuthorizationError::DeviceTokenMalformed => (
                StatusCode::UNAUTHORIZED,
                "Device Token is malformed and could not verify token.".to_string(),
            ),
            AuthorizationError::DeviceSignatureDecodeError => (
                StatusCode::UNAUTHORIZED,
                "Failed to decode device signature".to_string(),
            ),
            AuthorizationError::DeviceWrongSignature => (
                StatusCode::BAD_REQUEST,
                "Device signature did not match the expected signature".to_string(),
            ),
            AuthorizationError::DeviceLinkTooSlow => (
                StatusCode::FORBIDDEN,
                "Device link took too long".to_string(),
            ),
        }
        .into_response()
    }
}
