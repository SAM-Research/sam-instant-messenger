use axum::{extract::State, Json};

use axum_extra::{
    headers::{authorization::Basic, Authorization},
    TypedHeader,
};
use sam_common::api::account::{RegistrationRequest, RegistrationResponse};

use crate::{
    auth::authenticated_user::AuthenticatedUser,
    logic::account::{create_account, delete_account},
    state::{state_type::StateType, ServerState},
    ServerError,
};

/// Handle registration of new users
pub async fn account_register_endpoint<T: StateType>(
    State(mut state): State<ServerState<T>>,
    TypedHeader(Authorization(basic)): TypedHeader<Authorization<Basic>>,
    Json(req): Json<RegistrationRequest>,
) -> Result<Json<RegistrationResponse>, ServerError> {
    create_account(
        &mut state,
        req,
        basic.username().to_string(),
        basic.password().to_string(),
    )
    .await
    .map(Json)
}

// Handle deletion of account
pub async fn delete_account_endpoint<T: StateType>(
    State(mut state): State<ServerState<T>>,
    auth_user: AuthenticatedUser,
) -> Result<(), ServerError> {
    delete_account(&mut state, auth_user.account().id()).await
}
