


#[cfg(test)]
mod envelope_test{
    use std::collections::HashMap;
    use crate::{sam_message::{ClientEnvelope, ServerEnvelope}};
    use crate::sam_message::client_envelope::Type;

    #[test]
    fn client_envelope_test() {
        let envelope: ClientEnvelope = ClientEnvelope{
            r#type: Type::SignalMessage.into(),
            destination: "Magnus".to_string(),
            source: "Alex".to_string(),
            content: HashMap::from([
                (1, vec![10, 20, 30]),
                (2, vec![40, 50, 60]),
                (3, vec![70, 80, 90]),
            ]),
        };

        assert_eq!(envelope.r#type, Type::SignalMessage.into());
        assert_eq!(envelope.destination, "Magnus".to_string());
        assert_eq!(envelope.source, "Alex".to_string());
        assert_eq!(envelope.content.get(&1), Some(&vec![10, 20, 30]));
        assert_eq!(envelope.content.get(&2), Some(&vec![40, 50, 60]));
        assert_eq!(envelope.content.get(&3), Some(&vec![70, 80, 90]));
    }

    #[test]
    fn server_envelope_test() {
        let envelope: ServerEnvelope = ServerEnvelope{
            r#type: Type::SignalMessage.into(),
            destination: "Magnus".to_string(),
            source: "Alex".to_string(),
            content: vec![10, 20, 30],
        };

        assert_eq!(envelope.r#type, Type::SignalMessage.into());
        assert_eq!(envelope.destination, "Magnus".to_string());
        assert_eq!(envelope.source, "Alex".to_string());
        assert_eq!(envelope.content, vec![10, 20, 30]);
    }


}