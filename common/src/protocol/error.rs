use std::fmt::Display;

use derive_more::Error;

use crate::{
    address::DeviceId,
    sam_message::{error::Info, Error, ErrorCode},
};

#[derive(Debug, Error, Clone)]
pub enum ProtocolError {
    MessageIdDecode,
    NoEnvelopeInMessage,
    UnknownMessageAcked,
    NotEncryptedForAllDevices(#[error(not(source))] Vec<DeviceId>),
    EncryptedForExtraDevices(#[error(not(source))] Vec<DeviceId>),
}

impl From<ProtocolError> for Error {
    fn from(value: ProtocolError) -> Self {
        match value {
            ProtocolError::MessageIdDecode => Error {
                code: ErrorCode::MessageIdDecode.into(),
                message: "Could not decode Message ID".to_owned(),
                info: None,
            },
            ProtocolError::NoEnvelopeInMessage => Error {
                code: ErrorCode::NoEnvelopeInMessage.into(),
                message: "No Envelope in Message".to_owned(),
                info: None,
            },
            ProtocolError::UnknownMessageAcked => Error {
                code: ErrorCode::UnknownMessageAcked.into(),
                message: "Unknown Message was Acked by Client".to_owned(),
                info: None,
            },
            ProtocolError::NotEncryptedForAllDevices(vec) => Error {
                code: ErrorCode::NotEncryptedForAllDevices.into(),
                message: "Message was not encrypted for each of the recipient devices.".to_owned(),
                info: Some(Info::DeviceIds(vec.into())),
            },
            ProtocolError::EncryptedForExtraDevices(vec) => Error {
                code: ErrorCode::EncryptedForExtraDevices.into(),
                message: "Message was addressed to devices that do not exist.".to_owned(),
                info: Some(Info::DeviceIds(vec.into())),
            },
        }
    }
}

impl Display for ProtocolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let err: Error = self.to_owned().into();
        write!(f, "Error: code {} - '{}'", err.code, err.message)
    }
}
