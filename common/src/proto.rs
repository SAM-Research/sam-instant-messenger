use std::collections::HashMap;

use crate::{
    address::{AccountId, DeviceAddress, DeviceId, MessageId},
    sam_message::{ClientEnvelope, EnvelopeType, ServerEnvelope},
};

impl ClientEnvelope {
    pub fn new(
        r#type: EnvelopeType,
        recipient: AccountId,
        source: DeviceAddress,
        content: HashMap<DeviceId, Vec<u8>>,
    ) -> Self {
        Self {
            r#type: r#type.into(),
            destination_account_id: recipient.into(),
            source_account_id: source.account_id().into(),
            source_device_id: source.device_id().into(),
            content: content
                .into_iter()
                .map(|(id, bytes)| (id.into(), bytes))
                .collect(),
        }
    }
}

impl ServerEnvelope {
    pub fn new(
        r#type: EnvelopeType,
        destination: DeviceAddress,
        source: DeviceAddress,
        content: Vec<u8>,
        id: MessageId,
    ) -> Self {
        Self {
            r#type: r#type.into(),
            destination_account_id: destination.account_id().into(),
            destination_device_id: destination.device_id().into(),
            source_account_id: source.account_id().into(),
            source_device_id: source.device_id().into(),
            content,
            id: id.into(),
        }
    }
}

#[cfg(test)]
mod envelope_test {
    use crate::{
        address::{DeviceAddress, MessageId},
        sam_message::{
            ClientEnvelope, ClientMessage, EnvelopeType, MessageType, ServerEnvelope, ServerMessage,
        },
    };
    use std::collections::HashMap;
    use uuid::Uuid;

    #[test]
    fn client_message_test() {
        let ack_uuid = MessageId::generate();
        let ack: ClientMessage = ClientMessage {
            r#type: MessageType::Ack.into(),
            id: ack_uuid.clone().into(),
            message: None,
        };

        let id: Vec<u8> = ack_uuid.into();
        assert_eq!(ack.r#type, MessageType::Ack.into());
        assert_eq!(ack.id, id);
        assert_eq!(ack.message, None);

        let error_uuid = Uuid::new_v4().to_string();
        let error: ClientMessage = ClientMessage {
            r#type: MessageType::Error.into(),
            id: error_uuid.clone().into(),
            message: None,
        };

        let id: Vec<u8> = error_uuid.into();
        assert_eq!(error.r#type, MessageType::Error.into());
        assert_eq!(error.id, id);
        assert_eq!(error.message, None);

        let alice_address = DeviceAddress::random();
        let bob_address = DeviceAddress::random();

        let message_uuid = Uuid::new_v4().to_string();
        let envelope: ClientEnvelope = ClientEnvelope::new(
            EnvelopeType::SignalMessage.into(),
            alice_address.account_id(),
            bob_address.clone(),
            HashMap::from([
                (1.into(), vec![10, 20, 30]),
                (2.into(), vec![40, 50, 60]),
                (3.into(), vec![70, 80, 90]),
            ]),
        );
        let message: ClientMessage = ClientMessage {
            r#type: MessageType::Message.into(),
            id: message_uuid.clone().into(),
            message: Some(envelope.clone()),
        };

        let id: Vec<u8> = message_uuid.into();
        assert_eq!(message.r#type, MessageType::Message.into());
        assert_eq!(message.id, id);
        assert_eq!(message.message, Some(envelope.clone()));
        assert_eq!(envelope.r#type, EnvelopeType::SignalMessage.into());
        assert_eq!(
            alice_address.account_id(),
            envelope
                .destination_account_id
                .try_into()
                .expect("should be able to convert envelope account id to AccountId")
        );
        assert_eq!(
            bob_address.account_id(),
            envelope
                .source_account_id
                .try_into()
                .expect("should be able to convert envelope account id to AccountId"),
        );
        assert_eq!(envelope.content.get(&1), Some(&vec![10, 20, 30]));
        assert_eq!(envelope.content.get(&2), Some(&vec![40, 50, 60]));
        assert_eq!(envelope.content.get(&3), Some(&vec![70, 80, 90]));
    }

    #[test]
    fn server_message_test() {
        let uuid = MessageId::generate();
        let ack: ServerMessage = ServerMessage {
            r#type: MessageType::Ack.into(),
            id: uuid.clone().into(),
            message: None,
        };

        assert_eq!(ack.r#type, MessageType::Ack.into());
        assert_eq!(
            uuid,
            ack.id
                .try_into()
                .expect("should be able to convert envelope account id to MessageId")
        );
        assert_eq!(ack.message, None);

        let error_uuid = MessageId::generate();
        let error: ServerMessage = ServerMessage {
            r#type: MessageType::Error.into(),
            id: error_uuid.clone().into(),
            message: None,
        };

        assert_eq!(error.r#type, MessageType::Error.into());
        assert_eq!(
            error_uuid,
            error
                .id
                .try_into()
                .expect("should be able to convert envelope account id to MessageId")
        );
        assert_eq!(error.message, None);

        let alice_address = DeviceAddress::random();
        let bob_address = DeviceAddress::random();

        let message_uuid = MessageId::generate();
        let envelope: ServerEnvelope = ServerEnvelope::new(
            EnvelopeType::SignalMessage.into(),
            alice_address.clone(),
            bob_address.clone(),
            vec![10, 20, 30],
            message_uuid.clone().into(),
        );
        let message: ServerMessage = ServerMessage {
            r#type: MessageType::Message.into(),
            id: message_uuid.clone().into(),
            message: Some(envelope.clone()),
        };

        let id: Vec<u8> = message_uuid.into();
        assert_eq!(message.r#type, MessageType::Message.into());
        assert_eq!(message.id, id);
        assert_eq!(message.message, Some(envelope.clone()));
        assert_eq!(envelope.r#type, EnvelopeType::SignalMessage.into());
        assert_eq!(
            alice_address.account_id(),
            envelope
                .destination_account_id
                .try_into()
                .expect("should be able to convert envelope account id to MessageId")
        );
        assert_eq!(
            envelope.destination_device_id,
            alice_address.device_id().into()
        );
        assert_eq!(
            bob_address.account_id(),
            envelope
                .source_account_id
                .try_into()
                .expect("should be able to convert envelope account id to MessageId")
        );
        assert_eq!(envelope.content, vec![10, 20, 30]);
    }
}
