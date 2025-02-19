use axum::extract::ws::{CloseFrame, Message, WebSocket};
use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use log::{error, info};
use prost::Message as _;
use sam_common::sam_message::{ClientMessage, ServerMessage};
use tokio::sync::mpsc::{Receiver, Sender};
use uuid::Uuid;

use crate::{
    auth::authenticated_user::AuthenticatedUser,
    managers::traits::message_manager::MessageManager,
    state::{state_type::StateType, ServerState},
    ServerError,
};

use super::message::{handle_client_message, handle_server_envelope};

macro_rules! closing_err {
    ($username:expr, $err:expr) => {
        error!(
            "User '{}' websocket encountered an error '{}' closing connection...",
            $username, $err
        )
    };
}

pub async fn websocket_message_receiver<T: StateType>(
    mut state: ServerState<T>,
    mut receiver: SplitStream<WebSocket>,
    message_producer: Sender<Result<Option<ServerMessage>, ServerError>>,
    auth_user: AuthenticatedUser,
) {
    while let Some(Ok(msg)) = receiver.next().await {
        let decode_res = match msg {
            Message::Binary(b) => {
                info!(
                    "Received websocket message from user '{}'",
                    auth_user.account().username()
                );
                ClientMessage::decode(b).map_err(|_| ServerError::WebSocketDecodeError)
            }
            Message::Text(x) => {
                info!("{}", x);
                continue;
            }
            Message::Close(_) => Err(ServerError::WebSocketDisconnected),
            _ => continue,
        };

        let msg_res = match decode_res {
            Ok(msg) => handle_client_message(&mut state, &auth_user, msg).await,
            Err(e) => Err(e),
        };

        let is_msg_res_err = msg_res.is_err();
        if message_producer.send(msg_res).await.is_err() || is_msg_res_err {
            break;
        }
    }
}

pub async fn websocket_message_sender<T: StateType>(
    mut state: ServerState<T>,
    mut sender: SplitSink<WebSocket, Message>,
    mut message_consumer: Receiver<Result<Option<ServerMessage>, ServerError>>,
    auth_user: AuthenticatedUser,
) {
    while let Some(msg_res) = message_consumer.recv().await {
        let send_res = match msg_res {
            Ok(Some(msg)) => sender
                .send(Message::Binary(msg.encode_to_vec().into()))
                .await
                .map_err(|_| ServerError::WebSocketSendError),
            Err(ServerError::WebSocketDisconnected) => Err(ServerError::WebSocketDisconnected),
            Err(err) => {
                let res = sender
                    .send(Message::Close(Some(CloseFrame {
                        code: 1011,
                        reason: "Internal Server Error".into(),
                    })))
                    .await
                    .map_err(|_| ServerError::WebSocketSendError);
                match res {
                    Ok(_) => Err(err),
                    Err(x) => Err(x),
                }
            }
            Ok(None) => continue,
        };

        match send_res {
            Ok(_) => continue,
            Err(err) => {
                match err {
                    ServerError::WebSocketDisconnected => break,
                    _ => closing_err!(auth_user.account().username(), err),
                }
                break;
            }
        }
    }

    state
        .messages
        .unsubscribe(*auth_user.account().id(), auth_user.device().id())
        .await;
}

pub async fn websocket_dispatcher<T: StateType>(
    mut state: ServerState<T>,
    mut dispatch: Receiver<Uuid>,
    message_producer: Sender<Result<Option<ServerMessage>, ServerError>>,
    auth_user: AuthenticatedUser,
) {
    while let Some(msg_id) = dispatch.recv().await {
        let msg_res = state
            .messages
            .get_envelope(*auth_user.account().id(), auth_user.device().id(), msg_id)
            .await;

        let msg_res = match msg_res {
            Ok(envelope) => handle_server_envelope(&mut state, &auth_user, envelope).await,
            Err(e) => Err(e),
        };

        let is_msg_res_err = msg_res.is_err();
        if message_producer.send(msg_res).await.is_err() || is_msg_res_err {
            break;
        }
    }
}
