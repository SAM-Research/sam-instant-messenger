#[cfg(test)]
mod envelope_test {
    use crate::sam_message::{
        ClientEnvelope, ClientMessage, EnvelopeType, MessageType, ServerEnvelope, ServerMessage,
    };
    use std::collections::HashMap;
    use uuid::Uuid;

    #[test]
    fn client_message_test() {
        let ack_uuid = Uuid::new_v4().to_string();
        let ack: ClientMessage = ClientMessage {
            r#type: MessageType::Ack.into(),
            id: ack_uuid.clone(),
            message: None,
        };

        assert_eq!(ack.r#type, MessageType::Ack.into());
        assert_eq!(ack.id, ack_uuid);
        assert_eq!(ack.message, None);

        let error_uuid = Uuid::new_v4().to_string();
        let error: ClientMessage = ClientMessage {
            r#type: MessageType::Error.into(),
            id: error_uuid.clone(),
            message: None,
        };

        assert_eq!(error.r#type, MessageType::Error.into());
        assert_eq!(error.id, error_uuid);
        assert_eq!(error.message, None);

        let message_uuid = Uuid::new_v4().to_string();
        let envelope: ClientEnvelope = ClientEnvelope {
            r#type: EnvelopeType::SignalMessage.into(),
            destination: "Magnus".to_string(),
            source: "Alex".to_string(),
            content: HashMap::from([
                (1, vec![10, 20, 30]),
                (2, vec![40, 50, 60]),
                (3, vec![70, 80, 90]),
            ]),
        };
        let message: ClientMessage = ClientMessage {
            r#type: MessageType::Message.into(),
            id: message_uuid.clone(),
            message: Some(envelope.clone()),
        };

        assert_eq!(message.r#type, MessageType::Message.into());
        assert_eq!(message.id, message_uuid);
        assert_eq!(message.message, Some(envelope.clone()));
        assert_eq!(envelope.r#type, EnvelopeType::SignalMessage.into());
        assert_eq!(envelope.destination, "Magnus".to_string());
        assert_eq!(envelope.source, "Alex".to_string());
        assert_eq!(envelope.content.get(&1), Some(&vec![10, 20, 30]));
        assert_eq!(envelope.content.get(&2), Some(&vec![40, 50, 60]));
        assert_eq!(envelope.content.get(&3), Some(&vec![70, 80, 90]));
    }

    #[test]
    fn server_message_test() {
        let uuid = Uuid::new_v4().to_string();
        let ack: ServerMessage = ServerMessage {
            r#type: MessageType::Ack.into(),
            id: uuid.clone(),
            message: None,
        };

        assert_eq!(ack.r#type, MessageType::Ack.into());
        assert_eq!(ack.id, uuid);
        assert_eq!(ack.message, None);

        let error_uuid = Uuid::new_v4().to_string();
        let error: ServerMessage = ServerMessage {
            r#type: MessageType::Error.into(),
            id: error_uuid.clone(),
            message: None,
        };

        assert_eq!(error.r#type, MessageType::Error.into());
        assert_eq!(error.id, error_uuid);
        assert_eq!(error.message, None);

        let message_uuid = Uuid::new_v4().to_string();
        let envelope: ServerEnvelope = ServerEnvelope {
            r#type: EnvelopeType::SignalMessage.into(),
            destination: "Magnus".to_string(),
            source: "Alex".to_string(),
            content: vec![10, 20, 30],
            id: message_uuid.clone(),
        };
        let message: ServerMessage = ServerMessage {
            r#type: MessageType::Message.into(),
            id: message_uuid.clone(),
            message: Some(envelope.clone()),
        };

        assert_eq!(message.r#type, MessageType::Message.into());
        assert_eq!(message.id, message_uuid);
        assert_eq!(message.message, Some(envelope.clone()));
        assert_eq!(envelope.r#type, EnvelopeType::SignalMessage.into());
        assert_eq!(envelope.destination, "Magnus".to_string());
        assert_eq!(envelope.source, "Alex".to_string());
        assert_eq!(envelope.content, vec![10, 20, 30]);
        assert_eq!(envelope.id, message_uuid);
    }
}
