use axum::{http::StatusCode, response::IntoResponse};
use derive_more::derive::{Display, Error};
use log::debug;
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
    AccountNotExist,
    AccountExists,
    DeviceNotExist,
    DeviceExists,
    KeyNotExist,
    EnvelopeExists,
    EnvelopeNotExists,
    MessageSubscriberExists,
    MessageSubscriberNotExists,
    WebSocketDecodeError,
    WebSocketDisconnected,
    WebSocketSendError,
    MessageAlreadyPending,
    MessageNotPending,
    EnvelopeMalformed,
    MessageSubscriberSendErorr,
}

impl IntoResponse for ServerError {
    fn into_response(self) -> axum::response::Response {
        debug!("ServerError occured: {}", self);
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
            ServerError::AccountNotExist => todo!(),
            ServerError::AccountExists => todo!(),
            ServerError::DeviceNotExist => todo!(),
            ServerError::DeviceExists => todo!(),
            ServerError::KeyNotExist => todo!(),
            ServerError::EnvelopeExists => todo!(),
            ServerError::EnvelopeNotExists => todo!(),
            ServerError::MessageSubscriberExists => todo!(),
            ServerError::MessageSubscriberNotExists => todo!(),
            ServerError::WebSocketDecodeError => todo!(),
            ServerError::WebSocketDisconnected => todo!(),
            ServerError::WebSocketSendError => todo!(),
            ServerError::MessageAlreadyPending => todo!(),
            ServerError::MessageNotPending => todo!(),
            ServerError::EnvelopeMalformed => todo!(),
            ServerError::MessageSubscriberSendErorr => todo!(),
        }
        .into_response()
    }
}
