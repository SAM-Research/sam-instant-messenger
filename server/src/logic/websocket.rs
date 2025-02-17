use axum::extract::ws::{CloseFrame, Message, WebSocket};
use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use log::{error, info};
use prost::{bytes::Bytes, Message as _};
use sam_common::sam_message::{ClientMessage, ServerMessage};
use tokio::sync::mpsc::{Receiver, Sender};

use crate::{
    auth::authenticated_user::AuthenticatedUser,
    state::{state_type::StateType, ServerState},
    ServerError,
};

use super::message::handle_client_message;

pub async fn handle_receiver<T: StateType>(
    state: ServerState<T>,
    mut receiver: SplitStream<WebSocket>,
    message_sender: Sender<Result<ServerMessage, ServerError>>,
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
            Ok(msg) => handle_client_message(state.clone(), msg).await,
            Err(_) => Err(ServerError::WebSocketDecodeError),
        };

        let is_msg_res_err = msg_res.is_err();
        if message_sender.send(msg_res).await.is_err() || is_msg_res_err {
            break;
        }
    }
}

pub async fn handle_sender(
    mut sender: SplitSink<WebSocket, Message>,
    mut message_sender: Receiver<Result<ServerMessage, ServerError>>,
    auth_user: AuthenticatedUser,
) {
    while let Some(msg_res) = message_sender.recv().await {
        let send_res = match msg_res {
            Ok(msg) => sender
                .send(Message::Binary(msg.encode_to_vec()))
                .await
                .map_err(|_| ServerError::WebSocketSendError),
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
        };

        match send_res {
            Ok(_) => continue,
            Err(err) => {
                error!(
                    "User '{}' websocket encountered an error '{}' closing connection...",
                    auth_user.account().username(),
                    err
                );
                break;
            }
        }
    }
}
