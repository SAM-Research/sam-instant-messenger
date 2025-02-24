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

#[utoipa::path(
    post,
    path = "/api/v1/account",
    request_body(content = RegistrationRequest, content_type = "application/json"),
    responses(
        (status = 200, description = "Registration successful", body = RegistrationResponse, content_type = "application/json"),
        (status = 500, description = "Registration unsuccessful")
    ),
)]
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

/*
#[utoipa::path(
    delete,
    path = "/api/v1/account",
    responses(
        (status = 200, description = "Account deletion successful"),
        (status = 500, description = "Account deletion unsuccessful")
    ),
    params(
        ("auth_user" = AuthenticatedUser, Header)
    ),
)]

 */
pub async fn delete_account_endpoint<T: StateType>(
    State(mut state): State<ServerState<T>>,
    auth_user: AuthenticatedUser,
) -> Result<(), ServerError> {
    delete_account(&mut state, auth_user.account().id()).await
}
