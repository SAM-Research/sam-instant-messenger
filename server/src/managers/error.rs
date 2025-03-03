use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use derive_more::{Display, Error, From};
use log::error;

#[derive(Debug, From, Display, Error)]
pub enum AccountManagerError {
    AccountDoesNotExist,
    AccountAlreadyExists,
}

impl IntoResponse for AccountManagerError {
    fn into_response(self) -> Response {
        error!("AccountManagerError occurred {}", self);
        match self {
            AccountManagerError::AccountDoesNotExist => {
                (StatusCode::NOT_FOUND, "Account does not exist".to_string())
            }
            AccountManagerError::AccountAlreadyExists => {
                (StatusCode::CONFLICT, "Account already exists".to_string())
            }
        }
        .into_response()
    }
}

#[derive(Debug, From, Display, Error)]
pub enum DeviceManagerError {
    DeviceDoesNotExist,
    AccountDoesNotExist,
    DeviceAlreadyExists,
}

impl IntoResponse for DeviceManagerError {
    fn into_response(self) -> Response {
        error!("KeyManagerError occured {}", self);
        match self {
            DeviceManagerError::DeviceDoesNotExist => {
                (StatusCode::NOT_FOUND, "Device does not exist.".to_string())
            }
            DeviceManagerError::AccountDoesNotExist => {
                (StatusCode::NOT_FOUND, "Account does not exist.".to_string())
            }
            DeviceManagerError::DeviceAlreadyExists => {
                (StatusCode::BAD_REQUEST, "Device already exists".to_string())
            }
        }
        .into_response()
    }
}

#[derive(Debug, From, Display, Error)]
pub enum KeyManagerError {
    AccountDoesNotExist,
    KeyDoesNotExist,
}

impl IntoResponse for KeyManagerError {
    fn into_response(self) -> Response {
        error!("KeyManagerError occurred {}", self);
        match self {
            KeyManagerError::AccountDoesNotExist => {
                (StatusCode::NOT_FOUND, "Account does not exist.".to_string())
            }
            KeyManagerError::KeyDoesNotExist => {
                (StatusCode::NOT_FOUND, "Key does not exist.".to_string())
            }
        }
        .into_response()
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
                (StatusCode::CONFLICT, "Message already exists.".to_string())
            }
            MessageManagerError::MessageSubscriberSendError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Subscriber failed to send message.".to_string(),
            ),
            MessageManagerError::EnvelopeDoesNotExists => {
                (StatusCode::NOT_FOUND, "Message does not exist.".to_string())
            }
            MessageManagerError::AccountDoesNotExist => {
                (StatusCode::NOT_FOUND, "Could not find account.".to_string())
            }
            MessageManagerError::MessageSubscriberAlreadyExists => (
                StatusCode::CONFLICT,
                "Already subscribed to new messages.".to_string(),
            ),
            MessageManagerError::MessageSubscriberDoesNotExists => (
                StatusCode::NOT_FOUND,
                "Receiver not subscribed to new messages.".to_string(),
            ),
            MessageManagerError::MessageAlreadyPending => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Message is already pending.".to_string(),
            ),
            MessageManagerError::MessageNotPending => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Message not pending.".to_string(),
            ),
        }
        .into_response()
    }
}
