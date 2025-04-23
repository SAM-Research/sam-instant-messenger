use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use derive_more::{Display, Error, From};
use log::error;

use crate::auth::error::AuthorizationError;

#[derive(Debug, From, Display, Error)]
pub enum AccountManagerError {
    AccountDoesNotExist,
    AccountAlreadyExists,
    UsernameAlreadyExists,
    MalformedData,
    ServiceUnavailable,
}

impl IntoResponse for AccountManagerError {
    fn into_response(self) -> Response {
        error!("AccountManagerError occurred {}", self);
        match self {
            AccountManagerError::AccountDoesNotExist => {
                (StatusCode::NOT_FOUND, "Account does not exist")
            }
            AccountManagerError::AccountAlreadyExists => {
                (StatusCode::CONFLICT, "Account already exists")
            }
            AccountManagerError::UsernameAlreadyExists => {
                (StatusCode::CONFLICT, "Username already exists")
            }
            AccountManagerError::MalformedData => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Could not decode bytes from database",
            ),
            AccountManagerError::ServiceUnavailable => (StatusCode::SERVICE_UNAVAILABLE, ""),
        }
        .into_response()
    }
}

#[derive(Debug, From, Display, Error)]
pub enum DeviceManagerError {
    DeviceDoesNotExist,
    AccountDoesNotExist,
    NoDevicesFound,
    DeviceAlreadyExists,
    ServiceUnavailable,
}

impl IntoResponse for DeviceManagerError {
    fn into_response(self) -> Response {
        error!("KeyManagerError occured {}", self);
        match self {
            DeviceManagerError::DeviceDoesNotExist => {
                (StatusCode::NOT_FOUND, "Device does not exist.")
            }
            DeviceManagerError::AccountDoesNotExist => {
                (StatusCode::NOT_FOUND, "Account does not exist.")
            }
            DeviceManagerError::DeviceAlreadyExists => {
                (StatusCode::BAD_REQUEST, "Device already exists")
            }
            DeviceManagerError::ServiceUnavailable => (StatusCode::SERVICE_UNAVAILABLE, ""),
            DeviceManagerError::NoDevicesFound => {
                (StatusCode::NOT_FOUND, "Account has no devices.")
            }
        }
        .into_response()
    }
}

#[derive(Debug, From, Display, Error)]
pub enum KeyManagerError {
    AccountDoesNotExist,
    KeyDoesNotExist,
    Authorization(AuthorizationError),
    ServiceUnavailable,
}

impl IntoResponse for KeyManagerError {
    fn into_response(self) -> Response {
        error!("KeyManagerError occurred {}", self);
        match self {
            KeyManagerError::AccountDoesNotExist => {
                (StatusCode::NOT_FOUND, "Account does not exist.").into_response()
            }
            KeyManagerError::KeyDoesNotExist => {
                (StatusCode::NOT_FOUND, "Key does not exist.").into_response()
            }
            KeyManagerError::Authorization(err) => err.into_response(),
            KeyManagerError::ServiceUnavailable => StatusCode::SERVICE_UNAVAILABLE.into_response(),
        }
    }
}

#[derive(Debug, From, Display, Error)]
pub enum MessageManagerError {
    EnvelopeAlreadyExists,
    MessageSubscriberSendError,
    EnvelopeDoesNotExists,
    AccountDoesNotExist,
    MessageSubscriberAlreadyExists,
    MessageSubscriberDoesNotExists,
    MessageAlreadyPending,
    MessageNotPending,
}

impl IntoResponse for MessageManagerError {
    fn into_response(self) -> Response {
        error!("MessageManagerError occurred: {}", self);
        match self {
            MessageManagerError::EnvelopeAlreadyExists => {
                (StatusCode::CONFLICT, "Message already exists.")
            }
            MessageManagerError::MessageSubscriberSendError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Subscriber failed to send message.",
            ),
            MessageManagerError::EnvelopeDoesNotExists => {
                (StatusCode::NOT_FOUND, "Message does not exist.")
            }
            MessageManagerError::AccountDoesNotExist => {
                (StatusCode::NOT_FOUND, "Could not find account.")
            }
            MessageManagerError::MessageSubscriberAlreadyExists => {
                (StatusCode::CONFLICT, "Already subscribed to new messages.")
            }
            MessageManagerError::MessageSubscriberDoesNotExists => (
                StatusCode::NOT_FOUND,
                "Receiver not subscribed to new messages.",
            ),
            MessageManagerError::MessageAlreadyPending => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Message is already pending.",
            ),
            MessageManagerError::MessageNotPending => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Message not pending.")
            }
        }
        .into_response()
    }
}
