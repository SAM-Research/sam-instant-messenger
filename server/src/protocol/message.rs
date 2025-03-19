use std::collections::HashMap;

use crate::{
    auth::authenticated_user::AuthenticatedUser,
    managers::traits::{device_manager::DeviceManager, message_manager::MessageManager},
    state::{state_type::StateType, ServerState},
    ServerError,
};
use log::warn;
use sam_common::{
    address::{AccountId, DeviceId, MessageId},
    sam_message::{
        server_message::Content, ClientEnvelope, ClientMessage, ClientMessageType, DeviceList,
        ExtraDevicesStatus, MissingDevicesError, SamMessage, ServerEnvelope, ServerMessage,
        ServerMessageType,
    },
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
        ClientMessageType::ClientMessage => match message.message {
            Some(envelope) => Ok(Some(
                handle_client_envelope(state, auth_user, message_id, envelope).await?,
            )),
            None => Err(ServerError::EnvelopeMalformed),
        },
        ClientMessageType::ClientAck => {
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
    }
}

async fn handle_client_envelope<T: StateType>(
    state: &mut ServerState<T>,
    auth_user: &AuthenticatedUser,
    message_id: MessageId,
    envelope: ClientEnvelope,
) -> Result<ServerMessage, ServerError> {
    let mut dest_acc_ids = envelope
        .recipients()
        .ok_or(ServerError::EnvelopeMalformed)?;

    let sender_account_id = auth_user.account().id();
    let sender_device_id = auth_user.device().id();

    let mut extra_devices: HashMap<AccountId, Vec<DeviceId>> = HashMap::new();

    if dest_acc_ids.is_empty() {
        return Ok(ServerMessage::builder()
            .id(message_id.into())
            .r#type(ServerMessageType::EmptyMessage.into())
            .build());
    }

    if !dest_acc_ids.contains_key(&sender_account_id) {
        dest_acc_ids.insert(sender_account_id, Vec::new());
    }

    for (recipient, devices) in dest_acc_ids {
        let mut all_devices = state.devices.get_devices(recipient).await?;

        if recipient == sender_account_id {
            all_devices.retain(|id| *id != sender_device_id);
        }

        let recipient_id_as_vec = Into::<Vec<u8>>::into(recipient);

        let all_messages: HashMap<DeviceId, &SamMessage> = envelope
            .messages
            .iter()
            .filter(|message| message.destination_account_id == recipient_id_as_vec)
            .map(|message| (message.destination_device_id.into(), message))
            .collect();

        let missing_devices: Vec<DeviceId> = all_devices
            .clone()
            .into_iter()
            .filter(|id| !devices.contains(id))
            .collect();

        if !missing_devices.is_empty() {
            return Ok(ServerMessage::builder()
                .r#type(ServerMessageType::NotEncryptedForAllDevices.into())
                .content(Content::MissingDevices(MissingDevicesError {
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
                .source_account_id(sender_account_id.into())
                .source_device_id(sender_device_id.into())
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
            .r#type(ServerMessageType::EncryptedForExtraMessages.into())
            .content(Content::ExtraDevices(ExtraDevicesStatus {
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
        .r#type(ServerMessageType::ServerAck.into())
        .build())
}

pub async fn prepare_server_envelope<T: StateType>(
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
            .r#type(ServerMessageType::ServerMessage.into())
            .content(Content::ServerEnvelope(envelope))
            .build(),
    ))
}
