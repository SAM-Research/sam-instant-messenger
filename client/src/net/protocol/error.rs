use sam_common::DeviceId;

#[derive(Debug)]
pub enum ProtocolError {
    MalformedServerMessage,
    Disconnected,
    WrongServerAck,
    ExtraDevices(Vec<DeviceId>),
    MissingDevices(Vec<DeviceId>),
}

impl std::fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProtocolError::MalformedServerMessage => {
                write!(f, "ProtocolError::MalformedServerMessage")
            }
            ProtocolError::Disconnected => write!(f, "ProtocolError::Disconnected"),
            ProtocolError::WrongServerAck => write!(f, "ProtocolError::WrongServerAck"),
            ProtocolError::ExtraDevices(devices) => {
                write!(f, "ProtocolError::ExtraDevices({:?})", devices)
            }
            ProtocolError::MissingDevices(devices) => {
                write!(f, "ProtocolError::MissingDevices({:?})", devices)
            }
        }
    }
}
