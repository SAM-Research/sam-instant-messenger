use crate::state::state_type::StateType;
use crate::state::ServerState;
use crate::{auth::authenticated_user::AuthenticatedUser, ServerError};

use axum::extract::{State, WebSocketUpgrade};

// Handle creation of websocket connection to user
pub async fn websocket_endpoint<T: StateType>(
    State(_state): State<ServerState<T>>,
    _auth_user: AuthenticatedUser,
    _ws: WebSocketUpgrade,
) -> Result<axum::response::Response, ServerError> {
    todo!()
}
