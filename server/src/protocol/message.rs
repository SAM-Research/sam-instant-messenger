use crate::{
    auth::authenticated_user::AuthenticatedUser,
    managers::traits::{device_manager::DeviceManager, message_manager::MessageManager},
    state::{state_type::StateType, ServerState},
    ServerError,
};
use log::{error, warn};
use sam_common::{
    address::MessageId,
    sam_message::{ClientEnvelope, MessageType, Status, StatusCode},
};
use sam_common::{
    address::{AccountId, DeviceId},
    sam_message::{server_message::Content, ClientMessage, ServerEnvelope, ServerMessage},
};

pub async fn handle_client_message<T: StateType>(
    state: &mut ServerState<T>,
    auth_user: &AuthenticatedUser,
    message: ClientMessage,
) -> Result<Option<ServerMessage>, ServerError> {
    let message_id = match MessageId::try_from(message.id.clone()) {
        Ok(id) => id,
        Err(_) => return Err(ServerError::EnvelopeMalformed),
    };

    match message.r#type() {
        MessageType::Message => {
            if let Some(envelope) = message.message {
                Ok(Some(
                    handle_client_evelope(state, auth_user, message_id, envelope).await?,
                ))
            } else {
                Err(ServerError::EnvelopeMalformed)
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
                    Err(e)
                }
            }
        }
        MessageType::Status => {
            let account_id = auth_user.account().id();
            let device_id = auth_user.device().id();
            let pending_res = state
                .messages
                .remove_pending_message(account_id, device_id, message_id)
                .await;
            error!(
                "User '{}' failed to process message with id '{}'",
                auth_user.account().username(),
                message_id
            );
            match pending_res {
                Ok(_) => Ok(None),
                Err(e) => Err(e),
            }
        }
    }
}

async fn handle_client_evelope<T: StateType>(
    state: &mut ServerState<T>,
    auth_user: &AuthenticatedUser,
    message_id: MessageId,
    envelope: ClientEnvelope,
) -> Result<ServerMessage, ServerError> {
    let dest_id = match AccountId::try_from(envelope.destination_account_id.clone()) {
        Ok(id) => id,
        Err(_) => return Err(ServerError::EnvelopeMalformed),
    };

    let is_sync = auth_user.account().id() == dest_id;

    let (all_devices, needs_sync) = if is_sync {
        let devices = state
            .devices
            .get_devices(dest_id)
            .await?
            .iter()
            .filter(|id| **id != auth_user.device().id())
            .map(|id| (*id).clone())
            .collect();
        (devices, false)
    } else {
        let devices = state.devices.get_devices(dest_id).await?;

        let needs_sync = state
            .devices
            .get_devices(auth_user.account().id())
            .await?
            .len()
            > 1;
        (devices, needs_sync)
    };

    let missing_devices: Vec<DeviceId> = all_devices
        .clone()
        .into_iter()
        .filter(|id| !envelope.content.contains_key(id))
        .collect();

    if !missing_devices.is_empty() {
        return Ok(ServerMessage::builder()
            .r#type(MessageType::Status as i32)
            .content(Content::Status(
                Status::builder()
                    .code(StatusCode::NotEncryptedForAllDevices.into())
                    .device_ids(missing_devices.into())
                    .build(),
            ))
            .id(message_id.into())
            .build());
    }

    let extra_devices: Vec<DeviceId> = envelope
        .content
        .keys()
        .cloned()
        .filter(|id| !all_devices.contains(&(*id).into()))
        .map(|id| id.into())
        .collect();

    for (device_id, cipher) in envelope.content {
        let id = MessageId::generate();
        let server_envelope = ServerEnvelope::builder()
            .r#type(envelope.r#type)
            .destination_account_id(envelope.destination_account_id.clone())
            .destination_device_id(device_id)
            .source_account_id(envelope.source_account_id.clone())
            .source_device_id(envelope.source_device_id)
            .content(cipher)
            .id(id.into_bytes().to_vec())
            .build();
        state
            .messages
            .insert_envelope(dest_id, device_id.into(), id, server_envelope)
            .await?;
    }

    if !extra_devices.is_empty() {
        return Ok(ServerMessage::builder()
            .r#type(MessageType::Status as i32)
            .content(Content::Status(
                Status::builder()
                    .code(StatusCode::EncryptedForExtraDevices.into())
                    .device_ids(extra_devices.into())
                    .build(),
            ))
            .id(message_id.into())
            .build());
    }

    if needs_sync {
        return Ok(ServerMessage::builder()
            .r#type(MessageType::Status as i32)
            .content(Content::Status(
                Status::builder().code(StatusCode::NeedsSync.into()).build(),
            ))
            .id(message_id.into())
            .build());
    }

    Ok(ServerMessage::builder()
        .id(message_id.into())
        .r#type(MessageType::Ack.into())
        .build())
}

pub async fn handle_server_envelope<T: StateType>(
    state: &mut ServerState<T>,
    auth_user: &AuthenticatedUser,
    envelope: ServerEnvelope,
) -> Result<Option<ServerMessage>, ServerError> {
    let id = MessageId::try_from(envelope.id.clone())?;

    state
        .messages
        .add_pending_message(auth_user.account().id(), auth_user.device().id(), id)
        .await?;

    Ok(Some(
        ServerMessage::builder()
            .id(id.into())
            .r#type(MessageType::Message.into())
            .content(Content::Message(envelope))
            .build(),
    ))
}
