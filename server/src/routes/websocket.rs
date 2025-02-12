use crate::{
    auth::authenticated_user::AuthenticatedUser,
    logic::websocket::create_websocket,
    state::{traits::state_type::StateType, ServerState},
    ServerError,
};
use axum::extract::ws::WebSocket;
use axum::extract::{State, WebSocketUpgrade};
use futures_util::StreamExt;

// Handle creation of websocket connection to user
pub async fn websocket_endpoint<T: StateType<Socket = WebSocket>>(
    State(state): State<ServerState<T>>,
    auth_user: AuthenticatedUser,
    ws: WebSocketUpgrade,
) -> Result<axum::response::Response, ServerError> {
    Ok(ws.on_upgrade(|socket| async move {
        let (tx, rx) = socket.split();
        create_websocket(state, auth_user, tx, rx).await
    }))
}
