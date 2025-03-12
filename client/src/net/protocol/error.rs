use derive_more::Error;
use sam_common::sam_message::DeviceList;

#[derive(Debug, Error)]
pub enum ProtocolError {
    MalformedServerMessage,
    Disconnected,
    ExtraDevices(#[error(not(source))] Vec<DeviceList>),
    MissingDevices(#[error(not(source))] Vec<DeviceList>),
    WrongCertificatePath,
    FailedToExtractCertificate,
    InvalidCertificate,
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
            ProtocolError::WrongCertificatePath => {
                write!(f, "ProtocolError::WrongCertificatePath")
            }
            ProtocolError::FailedToExtractCertificate => {
                write!(f, "ProtocolError::FailedToExtractCertificate")
            }
            ProtocolError::InvalidCertificate => {
                write!(f, "ProtocolError::InvalidCertificate")
            }
        }
    }
}
