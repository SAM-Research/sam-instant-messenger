use axum::{
    extract::{State, WebSocketUpgrade},
    response::IntoResponse,
};
use futures_util::StreamExt;
use log::info;
use tokio::sync::mpsc;

use crate::{
    auth::authenticated_user::AuthenticatedUser,
    logic::websocket::{
        websocket_dispatcher, websocket_message_receiver, websocket_message_sender,
    },
    managers::traits::message_manager::MessageManager,
    state::{state_type::StateType, ServerState},
    ServerError,
};

pub async fn websocket_endpoint<T: StateType>(
    State(mut state): State<ServerState<T>>,
    auth_user: AuthenticatedUser,
    ws: WebSocketUpgrade,
) -> Result<impl IntoResponse, ServerError> {
    // TODO: find a solution to encapsulate this in a logic function
    let dispatch = state
        .messages
        .subscribe(*auth_user.account().id(), auth_user.device().id())
        .await?;

    Ok(ws.on_upgrade(move |socket| async move {
        info!("{} Connected!", auth_user.account().username());

        let (sender, receiver) = socket.split();
        let (msg_producer, msg_consumer) = mpsc::channel(5);

        tokio::spawn(websocket_message_receiver(
            state.clone(),
            receiver,
            msg_producer.clone(),
            auth_user.clone(),
        ));
        tokio::spawn(websocket_dispatcher(
            state.clone(),
            dispatch,
            msg_producer,
            auth_user.clone(),
        ));

        tokio::spawn(websocket_message_sender(
            state,
            sender,
            msg_consumer,
            auth_user,
        ));
    }))
}
