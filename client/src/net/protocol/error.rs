use sam_common::sam_message::DeviceList;

#[derive(Debug)]
pub enum ProtocolError {
    MalformedServerMessage,
    Disconnected,
    ExtraDevices(Vec<DeviceList>),
    MissingDevices(Vec<DeviceList>),
}

impl std::fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProtocolError::MalformedServerMessage => {
                write!(f, "ProtocolError::MalformedServerMessage")
            }
            ProtocolError::Disconnected => write!(f, "ProtocolError::Disconnected"),
            ProtocolError::ExtraDevices(devices) => {
                write!(f, "ProtocolError::ExtraDevices({:?})", devices)
            }
            ProtocolError::MissingDevices(devices) => {
                write!(f, "ProtocolError::MissingDevices({:?})", devices)
            }
        }
    }
}
