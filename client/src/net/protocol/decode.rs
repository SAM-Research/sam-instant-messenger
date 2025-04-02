use log::debug;
use sam_common::{
    address::MessageId,
    sam_message::{
        server_message::Content, DeviceList as ProtoDeviceList, ServerEnvelope, ServerMessage,
        ServerMessageType,
    },
    AccountId, DeviceId,
};

use super::error::DecodeError;

pub struct DeviceList {
    pub account_id: AccountId,
    pub devices: Vec<DeviceId>,
}

impl TryFrom<ProtoDeviceList> for DeviceList {
    type Error = DecodeError;

    fn try_from(value: ProtoDeviceList) -> Result<Self, Self::Error> {
        Ok(Self {
            account_id: AccountId::try_from(value.account_id)
                .inspect_err(|e| debug!("{e}"))
                .map_err(|_| DecodeError::MalformedServerMessage)?,
            devices: value.device_ids.iter().map(|id| (*id).into()).collect(),
        })
    }
}

pub enum EnvelopeOrStatus {
    Envelope(MessageId, ServerEnvelope),
    Status(ServerStatus),
}

impl TryFrom<ServerMessage> for EnvelopeOrStatus {
    type Error = DecodeError;

    fn try_from(value: ServerMessage) -> Result<Self, Self::Error> {
        let id = MessageId::try_from(value.id.clone())
            .inspect_err(|e| debug!("{e}"))
            .map_err(|_| DecodeError::MalformedServerMessage)?;

        let content = match value.r#type() {
            ServerMessageType::ServerAck => return Ok(ServerStatus::Ack(id).into()),
            ServerMessageType::EmptyMessage => return Ok(ServerStatus::EmptyMessage(id).into()),
            ServerMessageType::ServerMessage => {
                let content = value.content.ok_or(DecodeError::MalformedServerMessage)?;
                if !matches!(content, Content::ServerEnvelope(_)) {
                    return Err(DecodeError::MalformedServerMessage);
                }
                content
            }
            ServerMessageType::NotEncryptedForAllDevices => {
                let content = value.content.ok_or(DecodeError::MalformedServerMessage)?;
                if !matches!(content, Content::MissingDevices(_)) {
                    return Err(DecodeError::MalformedServerMessage);
                }
                content
            }
            ServerMessageType::EncryptedForExtraMessages => {
                let content = value.content.ok_or(DecodeError::MalformedServerMessage)?;
                if !matches!(content, Content::ExtraDevices(_)) {
                    return Err(DecodeError::MalformedServerMessage);
                }
                content
            }
        };
        Ok(match content {
            Content::ServerEnvelope(x) => EnvelopeOrStatus::Envelope(id, x),
            Content::MissingDevices(x) => ServerStatus::MissingDevices(
                id,
                x.device_lists
                    .into_iter()
                    .map(|li| li.try_into())
                    .collect::<Result<Vec<DeviceList>, _>>()?,
            )
            .into(),
            Content::ExtraDevices(x) => ServerStatus::ExtraDevices(
                id,
                x.device_lists
                    .into_iter()
                    .map(|li| li.try_into())
                    .collect::<Result<Vec<DeviceList>, _>>()?,
            )
            .into(),
        })
    }
}

pub enum MessageStatus {
    ExtraDevices(Vec<DeviceList>),
    MissingDevices(Vec<DeviceList>),
    Ok,
}

pub enum ServerStatus {
    Ack(MessageId),
    EmptyMessage(MessageId),
    MissingDevices(MessageId, Vec<DeviceList>),
    ExtraDevices(MessageId, Vec<DeviceList>),
}

impl ServerStatus {
    pub fn id(&self) -> MessageId {
        match self {
            ServerStatus::Ack(id)
            | ServerStatus::EmptyMessage(id)
            | ServerStatus::MissingDevices(id, _)
            | ServerStatus::ExtraDevices(id, _) => *id,
        }
    }

    pub fn validate(self, id: MessageId) -> Result<Option<MessageStatus>, DecodeError> {
        Ok(if id != self.id() {
            None
        } else {
            Some(self.try_into()?)
        })
    }
}

impl TryInto<MessageStatus> for ServerStatus {
    type Error = DecodeError;

    fn try_into(self) -> Result<MessageStatus, Self::Error> {
        Ok(match self {
            ServerStatus::Ack(_) => MessageStatus::Ok,
            ServerStatus::EmptyMessage(_) => return Err(DecodeError::EmptyMessage),
            ServerStatus::MissingDevices(_, device_lists) => {
                MessageStatus::MissingDevices(device_lists)
            }
            ServerStatus::ExtraDevices(_, device_lists) => {
                MessageStatus::ExtraDevices(device_lists)
            }
        })
    }
}

impl From<ServerStatus> for EnvelopeOrStatus {
    fn from(val: ServerStatus) -> Self {
        EnvelopeOrStatus::Status(val)
    }
}
