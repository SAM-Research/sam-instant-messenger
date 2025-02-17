use axum::{
    extract::{State, WebSocketUpgrade},
    response::IntoResponse,
};
use futures_util::StreamExt;
use log::info;
use tokio::sync::mpsc;

use crate::{
    auth::authenticated_user::AuthenticatedUser,
    logic::websocket::{handle_receiver, handle_sender},
    state::{state_type::StateType, ServerState},
};

pub async fn websocket_endpoint<T: StateType>(
    State(state): State<ServerState<T>>,
    auth_user: AuthenticatedUser,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| async move {
        info!("{} Connected!", auth_user.account().username());

        let (sender, receiver) = socket.split();

        let (tx, rx) = mpsc::channel(1);

        tokio::spawn(handle_sender(sender, rx, auth_user.clone()));
        tokio::spawn(handle_receiver(state, receiver, tx, auth_user));
    })
}
