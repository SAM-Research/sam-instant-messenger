use std::collections::HashMap;

use libsignal_protocol::CiphertextMessageType;

use crate::{
    address::DeviceId,
    sam_message::{ClientEnvelope, SamMessageType},
    AccountId,
};

impl From<CiphertextMessageType> for SamMessageType {
    fn from(value: CiphertextMessageType) -> Self {
        match value {
            CiphertextMessageType::Whisper => SamMessageType::SignalMessage,
            CiphertextMessageType::PreKey => SamMessageType::PreKeySignalMessage,
            CiphertextMessageType::SenderKey => SamMessageType::SenderKeyMessage,
            CiphertextMessageType::Plaintext => SamMessageType::PlaintextContent,
        }
    }
}

impl From<SamMessageType> for CiphertextMessageType {
    fn from(value: SamMessageType) -> Self {
        match value {
            SamMessageType::SignalMessage => CiphertextMessageType::Whisper,
            SamMessageType::PreKeySignalMessage => CiphertextMessageType::PreKey,
            SamMessageType::SenderKeyMessage => CiphertextMessageType::SenderKey,
            SamMessageType::PlaintextContent => CiphertextMessageType::Plaintext,
        }
    }
}

impl ClientEnvelope {
    pub fn recipients(&self) -> Option<HashMap<AccountId, Vec<DeviceId>>> {
        let mut devices: HashMap<AccountId, Vec<DeviceId>> = HashMap::new();

        for msg in self.messages.iter() {
            devices
                .entry(msg.destination_account_id.clone().try_into().ok()?)
                .or_default()
                .push(msg.destination_device_id.into());
        }
        Some(devices)
    }
}
