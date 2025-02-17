use crate::{
    auth::authenticated_user::AuthenticatedUser,
    managers::traits::message_manager::MessageManager,
    state::{state_type::StateType, ServerState},
    ServerError,
};
use log::error;
use sam_common::sam_message::{ClientEnvelope, MessageType};
use sam_common::sam_message::{ClientMessage, ServerEnvelope, ServerMessage};
use uuid::Uuid;

macro_rules! error_message {
    ($msg_id:expr) => {
        Ok(Some(
            ServerMessage::builder()
                .r#type(MessageType::Error as i32)
                .id($msg_id)
                .build(),
        ))
    };
}

pub async fn handle_client_message<T: StateType>(
    state: &mut ServerState<T>,
    auth_user: &AuthenticatedUser,
    message: ClientMessage,
) -> Result<Option<ServerMessage>, ServerError> {
    let message_id = match message.id.parse() {
        Ok(id) => id,
        Err(_) => return error_message!(message.id),
    };

    match message.r#type() {
        MessageType::Message => {
            if let Some(envelope) = message.message {
                handle_client_evelope(state, message.id, envelope).await
            } else {
                error_message!(message.id)
            }
        }
        MessageType::Ack => {
            let pending_res = state
                .messages
                .remove_pending_message(
                    *auth_user.account().id(),
                    auth_user.device().id(),
                    message_id,
                )
                .await;

            match pending_res {
                Ok(_) => Ok(None),
                Err(_) => error_message!(message.id),
            }
        }
        MessageType::Error => {
            // TODO: scheduler to remove error messages from pending?
            error!(
                "User '{}' failed to process message with id '{}'",
                auth_user.account().username(),
                message_id
            );
            Ok(None)
        }
    }
}

async fn handle_client_evelope<T: StateType>(
    state: &mut ServerState<T>,
    message_id: String,
    envelope: ClientEnvelope,
) -> Result<Option<ServerMessage>, ServerError> {
    let dest_id = match envelope.destination.parse() {
        Ok(id) => id,
        Err(_) => return error_message!(message_id),
    };

    for (device_id, cipher) in envelope.content {
        let id = Uuid::new_v4();
        let server_envelope = ServerEnvelope::builder()
            .r#type(envelope.r#type)
            .destination(envelope.destination.clone())
            .source(envelope.source.clone())
            .content(cipher)
            .id(id.to_string())
            .build();
        state
            .messages
            .insert_envelope(dest_id, device_id, id, server_envelope)
            .await?;
    }

    Ok(Some(
        ServerMessage::builder()
            .id(message_id)
            .r#type(MessageType::Ack as i32)
            .build(),
    ))
}

pub async fn handle_server_envelope<T: StateType>(
    state: &mut ServerState<T>,
    auth_user: &AuthenticatedUser,
    envelope: ServerEnvelope,
) -> Result<Option<ServerMessage>, ServerError> {
    let id = match envelope.id.parse() {
        Ok(id) => id,
        Err(_) => return Err(ServerError::EnvelopeMalformed),
    };

    state
        .messages
        .add_pending_message(*auth_user.account().id(), auth_user.device().id(), id)
        .await?;

    let id = envelope.id.clone();
    Ok(Some(
        ServerMessage::builder()
            .id(id)
            .message(envelope)
            .r#type(MessageType::Message as i32)
            .build(),
    ))
}
