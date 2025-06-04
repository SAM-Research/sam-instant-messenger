use axum::extract::ws::{CloseFrame, Message, WebSocket};
use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use log::{debug, error, info};
use prost::Message as _;
use sam_common::{
    address::MessageId,
    sam_message::{ClientMessage, ServerMessage},
};
use tokio::select;
use tokio::sync::mpsc::Receiver;

use crate::protocol::message::{handle_client_message, prepare_server_envelope};
use crate::{
    auth::authenticated_user::AuthenticatedUser,
    managers::traits::message_manager::MessageManager,
    state::{state_type::StateType, ServerState},
};
use crate::{
    protocol::error::{WebSocketError, WebSocketSessionError},
    ServerError,
};

pub async fn init_websocket<T: StateType>(
    state: ServerState<T>,
    auth_user: AuthenticatedUser,
    socket: WebSocket,
    dispatch: Receiver<MessageId>,
) {
    info!("{} Connected!", auth_user.account().username());
    let (sender, receiver) = socket.split();

    tokio::spawn(websocket_message_receiver(
        state.clone(),
        receiver,
        dispatch,
        sender,
        auth_user,
    ));
}

async fn websocket_message_receiver<T: StateType>(
    mut state: ServerState<T>,
    mut receiver: SplitStream<WebSocket>,
    mut dispatch: Receiver<MessageId>,
    mut sender: SplitSink<WebSocket, Message>,
    auth_user: AuthenticatedUser,
) {
    debug!(
        "Started WS handler for user '{}'",
        auth_user.account().username()
    );
    loop {
        let res = select! {
            msg = receiver.next() => {
                if let Some(Ok(msg)) = msg {
                    match handle_message_received(&mut state, &auth_user, msg).await {
                        None => continue,
                        Some(msg_res) => msg_res
                    }
                }
                else {
                    info!("User '{}' Disconnected", auth_user.account().username());
                    break
                }
            },
            msg_id = dispatch.recv() => {
                if let Some(id) = msg_id {
                    message_dispatched(&mut state, &auth_user, id).await
                } else {
                    info!("User '{}' Unsubscribed from message dispatcher", auth_user.account().username());
                    break
                }
            }
        };

        let msg = match res {
            Ok(Some(msg)) => msg,
            Ok(None) => continue,
            Err(WebSocketSessionError::WebSocket(WebSocketError::WebSocketDisconnected)) => {
                break;
            }
            Err(e) => {
                error!("Websocket error: {e}");
                error!(
                    "Closing connection for '{}'",
                    auth_user.account().username()
                );
                let _ = sender
                    .send(Message::Close(Some(CloseFrame {
                        code: 1011,
                        reason: "Internal Server Error".into(),
                    })))
                    .await
                    .inspect_err(|e| debug!("{e}"));
                break;
            }
        };
        if sender
            .send(Message::Binary(msg.encode_to_vec().into()))
            .await
            .inspect_err(|e| debug!("{e}"))
            .is_err()
        {
            debug!("User disconnected");
            break;
        }
    }
    state
        .messages
        .unsubscribe(auth_user.account().id(), auth_user.device().id())
        .await;
    debug!(
        "Stopped WS handler for user '{}'",
        auth_user.account().username()
    );
}

async fn handle_message_received<T: StateType>(
    state: &mut ServerState<T>,
    auth_user: &AuthenticatedUser,
    msg: Message,
) -> Option<Result<Option<ServerMessage>, WebSocketSessionError>> {
    let decode_res = match msg {
        Message::Binary(b) => {
            info!(
                "Received websocket message from user '{}'",
                auth_user.account().username()
            );
            ClientMessage::decode(b)
                .inspect_err(|e| debug!("{e}"))
                .map_err(|_| WebSocketError::WebSocketDecodeError)
        }
        Message::Close(_) => Err(WebSocketError::WebSocketDisconnected),
        _ => return None,
    };

    let msg_res = match decode_res {
        Ok(msg) => handle_client_message(state, auth_user, msg)
            .await
            .map_err(WebSocketSessionError::from),
        Err(e) => Err(WebSocketSessionError::from(e)),
    };
    Some(msg_res)
}

async fn message_dispatched<T: StateType>(
    state: &mut ServerState<T>,
    auth_user: &AuthenticatedUser,
    msg_id: MessageId,
) -> Result<Option<ServerMessage>, WebSocketSessionError> {
    debug!(
        "Dispatching message to user '{}'",
        auth_user.account().username()
    );
    let msg_res = state
        .messages
        .get_envelope(auth_user.account().id(), auth_user.device().id(), msg_id)
        .await;

    match msg_res {
        Ok(envelope) => prepare_server_envelope(state, auth_user, envelope)
            .await
            .map_err(WebSocketSessionError::from),
        Err(e) => Err(WebSocketSessionError::from(ServerError::from(e))),
    }
}
