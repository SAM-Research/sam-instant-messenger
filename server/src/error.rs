use axum::{http::StatusCode, response::IntoResponse};
use derive_more::derive::{Display, Error, From};
use sam_common::LibError;

pub type Result<T> = std::result::Result<T, ServerError>;

#[derive(Debug, Display, Error, From)]
pub enum ServerError {
    #[error(ignore)]
    Custom(String),
    Lib(LibError),
    DeviceAlreadyRegistered,
    #[error(ignore)]
    HKDFOutputTooLong(u32),
    KeyVerification,
}

#[derive(Debug, Clone)]
pub struct HTTPError {
    pub status_code: StatusCode,
    pub body: String,
}

impl IntoResponse for HTTPError {
    fn into_response(self) -> axum::response::Response {
        let status_code = self.status_code;
        (status_code, self.body).into_response()
    }
}
