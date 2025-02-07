use axum::{extract::State, Json};
use sam_common::api::account::{RegistrationRequest, RegistrationResponse};

use crate::{error::HTTPError, state::ServerState};

/// Handle registration of new users
#[axum::debug_handler]
pub async fn account_register_endpoint(
    State(state): State<ServerState>,
    Json(registration): Json<RegistrationRequest>,
) -> Result<Json<RegistrationResponse>, HTTPError> {
    todo!()
}

// Handle deletion of account
pub async fn delete_account_endpoint(State(state): State<ServerState>) -> Result<(), HTTPError> {
    todo!()
}
