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
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::protocol::error::{WebSocketError, WebSocketSessionError};
use crate::protocol::message::{handle_client_message, prepare_server_envelope};
use crate::{
    auth::authenticated_user::AuthenticatedUser,
    managers::traits::message_manager::MessageManager,
    state::{state_type::StateType, ServerState},
};

macro_rules! closing_err {
    ($username:expr, $err:expr) => {
        error!(
            "User '{}' websocket encountered an error '{}' closing connection...",
            $username, $err
        )
    };
}

pub async fn init_websocket<T: StateType>(
    state: ServerState<T>,
    auth_user: AuthenticatedUser,
    socket: WebSocket,
    dispatch: Receiver<MessageId>,
) {
    info!("{} Connected!", auth_user.account().username());
    let (sender, receiver) = socket.split();
    let (msg_producer, msg_consumer) = mpsc::channel(state.messages.channel_buffer().await);

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
}

async fn websocket_message_receiver<T: StateType>(
    mut state: ServerState<T>,
    mut receiver: SplitStream<WebSocket>,
    message_producer: Sender<Result<Option<ServerMessage>, WebSocketSessionError>>,
    auth_user: AuthenticatedUser,
) {
    while let Some(Ok(msg)) = receiver.next().await {
        let decode_res = match msg {
            Message::Binary(b) => {
                info!(
                    "Received websocket message from user '{}'",
                    auth_user.account().username()
                );
                ClientMessage::decode(b).map_err(|_| WebSocketError::WebSocketDecodeError)
            }
            Message::Close(_) => Err(WebSocketError::WebSocketDisconnected),
            _ => continue,
        };

        let msg_res = match decode_res {
            Ok(msg) => handle_client_message(&mut state, &auth_user, msg)
                .await
                .map_err(WebSocketSessionError::from),
            Err(e) => Err(WebSocketSessionError::from(e)),
        };

        let is_msg_res_err = msg_res.is_err();
        if message_producer.send(msg_res).await.is_err() || is_msg_res_err {
            break;
        }
    }
}

async fn websocket_message_sender<T: StateType>(
    mut state: ServerState<T>,
    mut sender: SplitSink<WebSocket, Message>,
    mut message_consumer: Receiver<Result<Option<ServerMessage>, WebSocketSessionError>>,
    auth_user: AuthenticatedUser,
) {
    while let Some(msg_res) = message_consumer.recv().await {
        let send_res = match msg_res {
            Ok(Some(msg)) => {
                debug!("Sending message to '{}'", auth_user.account().username());
                sender
                    .send(Message::Binary(msg.encode_to_vec().into()))
                    .await
                    .map_err(|_| WebSocketSessionError::from(WebSocketError::WebSocketSendError))
            }
            Err(WebSocketSessionError::WebSocket(WebSocketError::WebSocketDisconnected)) => Err(
                WebSocketSessionError::from(WebSocketError::WebSocketDisconnected),
            ),
            Err(err) => {
                let res = sender
                    .send(Message::Close(Some(CloseFrame {
                        code: 1011,
                        reason: "Internal Server Error".into(),
                    })))
                    .await
                    .map_err(|_| WebSocketError::WebSocketSendError);
                match res {
                    Ok(_) => Err(err),
                    Err(x) => Err(WebSocketSessionError::from(x)),
                }
            }
            Ok(None) => continue,
        };

        match send_res {
            Ok(_) => continue,
            Err(err) => {
                match err {
                    WebSocketSessionError::WebSocket(WebSocketError::WebSocketDisconnected) => {
                        break;
                    }
                    _ => closing_err!(auth_user.account().username(), err),
                }
                break;
            }
        }
    }
    state
        .messages
        .unsubscribe(auth_user.account().id(), auth_user.device().id())
        .await;
}

async fn websocket_dispatcher<T: StateType>(
    mut state: ServerState<T>,
    mut dispatch: Receiver<MessageId>,
    message_producer: Sender<Result<Option<ServerMessage>, WebSocketSessionError>>,
    auth_user: AuthenticatedUser,
) {
    while let Some(msg_id) = dispatch.recv().await {
        debug!(
            "Dispatching message to user '{}'",
            auth_user.account().username()
        );
        let msg_res = state
            .messages
            .get_envelope(auth_user.account().id(), auth_user.device().id(), msg_id)
            .await;

        let msg_res = match msg_res {
            Ok(envelope) => prepare_server_envelope(&mut state, &auth_user, envelope)
                .await
                .map_err(WebSocketSessionError::from),
            Err(e) => Err(WebSocketSessionError::from(e)),
        };

        let is_msg_res_err = msg_res.is_err();
        if message_producer.send(msg_res).await.is_err() || is_msg_res_err {
            break;
        }
    }
}
