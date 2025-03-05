use std::collections::HashMap;

use crate::{
    auth::authenticated_user::AuthenticatedUser,
    managers::traits::{device_manager::DeviceManager, message_manager::MessageManager},
    state::{state_type::StateType, ServerState},
    ServerError,
};
use log::{error, warn};
use sam_common::{
    address::MessageId,
    address::{AccountId, DeviceId},
    sam_message::{server_message::Content, ClientMessage, ServerEnvelope, ServerMessage},
    sam_message::{ClientEnvelope, DeviceList, MessageType, SamMessage, Status, StatusCode},
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
                    handle_client_envelope(
                        state,
                        auth_user,
                        message_id,
                        envelope,
                        auth_user.account().id(),
                        auth_user.device().id(),
                    )
                    .await?,
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

async fn handle_client_envelope<T: StateType>(
    state: &mut ServerState<T>,
    auth_user: &AuthenticatedUser,
    message_id: MessageId,
    envelope: ClientEnvelope,
    source_account_id: AccountId,
    source_device_id: DeviceId,
) -> Result<ServerMessage, ServerError> {
    let dest_acc_ids = envelope
        .recipients()
        .ok_or(ServerError::EnvelopeMalformed)?;

    let sender_acc = auth_user.account().id();
    let sender_dev = auth_user.device().id();

    let is_sync = dest_acc_ids.contains_key(&auth_user.account().id());
    let needs_sync = !is_sync && state.devices.get_devices(sender_acc).await?.len() > 1;

    let mut extra_devices: HashMap<AccountId, Vec<DeviceId>> = HashMap::new();
    for (recipient, devices) in dest_acc_ids {
        let all_devices = state.devices.get_devices(recipient).await?;
        let all_messages: HashMap<DeviceId, &SamMessage> = envelope
            .messages
            .iter()
            .filter(|message| message.destination_account_id == Into::<Vec<u8>>::into(recipient))
            .map(|message| (message.destination_device_id.into(), message))
            .collect();

        let missing_devices: Vec<DeviceId> = all_devices
            .clone()
            .into_iter()
            .filter(|id| !devices.contains(id))
            .collect();

        if !missing_devices.is_empty() {
            return Ok(ServerMessage::builder()
                .r#type(MessageType::Status as i32)
                .content(Content::Status(Status {
                    code: StatusCode::NotEncryptedForAllDevices.into(),
                    device_lists: vec![DeviceList {
                        account_id: recipient.into(),
                        device_ids: missing_devices.into_iter().map(|id| id.into()).collect(),
                    }],
                }))
                .id(message_id.into())
                .build());
        }

        extra_devices.insert(
            recipient,
            devices
                .into_iter()
                .filter(|id| !all_devices.contains(id))
                .collect(),
        );

        for (device_id, message) in all_messages {
            let id = MessageId::generate();
            let server_envelope = ServerEnvelope::builder()
                .r#type(message.r#type)
                .destination_account_id(message.destination_account_id.clone())
                .destination_device_id(message.destination_device_id)
                .source_account_id(source_account_id.into())
                .source_device_id(source_device_id.into())
                .content(message.content.clone())
                .id(id.into_bytes().to_vec())
                .build();
            state
                .messages
                .insert_envelope(recipient, device_id, id, server_envelope)
                .await?;
        }
    }
    if !extra_devices.iter().all(|(_, list)| list.is_empty()) {
        return Ok(ServerMessage::builder()
            .r#type(MessageType::Status as i32)
            .content(Content::Status(Status {
                code: StatusCode::EncryptedForExtraDevices.into(),
                device_lists: extra_devices
                    .into_iter()
                    .map(|(account_id, device_ids)| DeviceList {
                        account_id: account_id.into(),
                        device_ids: device_ids.into_iter().map(|id| id.into()).collect(),
                    })
                    .collect(),
            }))
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
