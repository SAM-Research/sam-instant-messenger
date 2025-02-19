use crate::{
    auth::authenticated_user::AuthenticatedUser,
    managers::traits::message_manager::MessageManager,
    state::{state_type::StateType, ServerState},
    ServerError,
};
use log::{error, warn};
use sam_common::{
    address::AccountId,
    sam_message::{ClientMessage, ServerEnvelope, ServerMessage},
};
use sam_common::{
    address::MessageId,
    sam_message::{ClientEnvelope, MessageType},
};

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
    let message_id = match MessageId::try_from(message.id.clone()) {
        Ok(id) => id,
        Err(_) => return error_message!(message.id),
    };

    match message.r#type() {
        MessageType::Message => {
            if let Some(envelope) = message.message {
                handle_client_evelope(state, message_id, envelope).await
            } else {
                error_message!(message_id.into())
            }
        }
        MessageType::Ack => {
            let account_id = auth_user.account().id();
            let device_id = auth_user.device().id();
            let pending_res = state
                .messages
                .remove_pending_message(account_id, device_id, message_id)
                .await;
            match pending_res {
                Ok(_) => {
                    let remove_res = state
                        .messages
                        .remove_envelope(account_id, device_id, message_id)
                        .await;
                    match remove_res {
                        Ok(_) => Ok(None),
                        Err(e) => Err(e),
                    }
                }
                Err(e) => {
                    warn!(
                        "error '{}', websocket user '{}' sent an ack with unknown id",
                        e,
                        auth_user.account().username()
                    );
                    error_message!(message_id.into())
                }
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
    message_id: MessageId,
    envelope: ClientEnvelope,
) -> Result<Option<ServerMessage>, ServerError> {
    let dest_id = match AccountId::try_from(envelope.destination_account_id.clone()) {
        Ok(id) => id,
        Err(_) => return error_message!(message_id.into()),
    };

    for (device_id, cipher) in envelope.content {
        let id = MessageId::generate();
        let server_envelope = ServerEnvelope::builder()
            .r#type(envelope.r#type)
            .destination_account_id(envelope.destination_account_id.clone())
            .destination_device_id(device_id)
            .source_account_id(envelope.source_account_id.clone())
            .source_device_id(envelope.source_device_id.clone())
            .content(cipher)
            .id(id.into_bytes().to_vec())
            .build();
        state
            .messages
            .insert_envelope(dest_id, device_id.into(), id, server_envelope)
            .await?;
    }

    Ok(Some(
        ServerMessage::builder()
            .id(message_id.into())
            .r#type(MessageType::Ack as i32)
            .build(),
    ))
}

pub async fn handle_server_envelope<T: StateType>(
    state: &mut ServerState<T>,
    auth_user: &AuthenticatedUser,
    envelope: ServerEnvelope,
) -> Result<Option<ServerMessage>, ServerError> {
    let id = match MessageId::try_from(envelope.id.clone()) {
        Ok(id) => id,
        Err(_) => return Err(ServerError::EnvelopeMalformed),
    };

    state
        .messages
        .add_pending_message(auth_user.account().id(), auth_user.device().id(), id)
        .await?;

    Ok(Some(
        ServerMessage::builder()
            .id(id.into())
            .message(envelope)
            .r#type(MessageType::Message as i32)
            .build(),
    ))
}
