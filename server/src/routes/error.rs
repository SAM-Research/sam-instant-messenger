use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use derive_more::{Display, Error, From};

#[derive(Debug, Display, Error, From)]
pub enum RouterError {
    DeviceProvisionUnAuth,
    DeviceUnAuth,
    NoDeviceIdsInRequest,
}

impl IntoResponse for RouterError {
    fn into_response(self) -> Response {
        match self {
            RouterError::DeviceProvisionUnAuth => (
                StatusCode::FORBIDDEN,
                "Only main device can request provision".to_string(),
            ),
            RouterError::DeviceUnAuth => (
                StatusCode::FORBIDDEN,
                "The main device cannot be unlinked.".to_string(),
            ),
            RouterError::NoDeviceIdsInRequest => (
                StatusCode::BAD_REQUEST,
                "No device ids were supplied in the request".to_string(),
            ),
        }
        .into_response()
    }
}
