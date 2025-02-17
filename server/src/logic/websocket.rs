use axum::extract::ws::{CloseFrame, Message, WebSocket};
use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use log::{error, info};
use prost::{bytes::Bytes, Message as _};
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
        );
    };
}

pub async fn websocket_message_receiver<T: StateType>(
    mut state: ServerState<T>,
    mut receiver: SplitStream<WebSocket>,
    message_producer: Sender<Result<ServerMessage, ServerError>>,
    auth_user: AuthenticatedUser,
) {
    while let Some(Ok(msg)) = receiver.next().await {
        info!(
            "Received websocket message from user '{}'",
            auth_user.account().username()
        );
        let decode_res = match msg {
            Message::Binary(b) => {
                ClientMessage::decode(Bytes::from(b)).map_err(|_| ServerError::WebSocketDecodeError)
            }
            Message::Close(_) => Err(ServerError::WebSocketDisconnected),
            _ => continue,
        };

        let msg_res = match decode_res {
            Ok(msg) => handle_client_message(&mut state, msg).await,
            Err(_) => Err(ServerError::WebSocketDecodeError),
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
    mut message_consumer: Receiver<Result<ServerMessage, ServerError>>,
    auth_user: AuthenticatedUser,
) {
    while let Some(msg_res) = message_consumer.recv().await {
        let send_res = match msg_res {
            Ok(msg) => sender
                .send(Message::Binary(msg.encode_to_vec()))
                .await
                .map_err(|_| ServerError::WebSocketSendError),
            Err(err) => {
                closing_err!(auth_user.account().username(), err);
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
        };

        match send_res {
            Ok(_) => continue,
            Err(err) => {
                closing_err!(auth_user.account().username(), err);
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
    message_producer: Sender<Result<ServerMessage, ServerError>>,
    auth_user: AuthenticatedUser,
) {
    while let Some(msg_id) = dispatch.recv().await {
        let msg_res = state
            .messages
            .get_message(*auth_user.account().id(), auth_user.device().id(), msg_id)
            .await;

        let msg_res = match msg_res {
            Ok(envelope) => handle_server_envelope(&mut state, envelope).await,
            Err(e) => Err(e),
        };

        let is_msg_res_err = msg_res.is_err();
        if message_producer.send(msg_res).await.is_err() || is_msg_res_err {
            break;
        }
    }
}
