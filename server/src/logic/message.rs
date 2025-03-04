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
    sam_message::{ClientEnvelope, DeviceList, Error, ErrorCode, MessageType, SamMessage},
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
                    handle_client_envelope(
                        state,
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
        MessageType::Error => {
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
    message_id: MessageId,
    envelope: ClientEnvelope,
    source_account_id: AccountId,
    source_device_id: DeviceId,
) -> Result<ServerMessage, ServerError> {
    let dest_acc_ids = envelope
        .recipients()
        .ok_or(ServerError::EnvelopeMalformed)?;

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
                .r#type(MessageType::Error as i32)
                .content(Content::Error(Error {
                    code: ErrorCode::NotEncryptedForAllDevices.into(),
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
            .r#type(MessageType::Error as i32)
            .content(Content::Error(Error {
                code: ErrorCode::EncryptedForExtraDevices.into(),
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
    let id =
        MessageId::try_from(envelope.id.clone()).map_err(|_| ServerError::EnvelopeMalformed)?;

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
