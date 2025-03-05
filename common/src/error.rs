use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use derive_more::derive::{Display, Error};

pub type Result<T> = std::result::Result<T, LibError>;

#[derive(Debug, Display, Error)]
pub enum LibError {
    #[error(ignore)]
    Custom(String),
    #[error(ignore)]
    RegistrationKeyFieldsRequired,
    ConversionError,
}

impl IntoResponse for LibError {
    fn into_response(self) -> Response {
        match self {
            LibError::Custom(string) => (StatusCode::INTERNAL_SERVER_ERROR, string),
            LibError::RegistrationKeyFieldsRequired => (
                StatusCode::BAD_REQUEST,
                "Registration key should be added in the request.".to_string(),
            ),
            LibError::ConversionError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to convert".to_string(),
            ),
        }
        .into_response()
    }
}
