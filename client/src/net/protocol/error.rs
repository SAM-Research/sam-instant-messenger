use sam_common::{sam_message, AccountId, DeviceId};

#[derive(Debug)]
pub enum ProtocolError {
    MalformedServerMessage,
    Disconnected,
    ExtraDevices(Vec<DeviceList>),
    MissingDevices(Vec<DeviceList>),
}

#[derive(Debug)]
pub struct DeviceList {
    _account_id: AccountId,
    _device_ids: Vec<DeviceId>,
}

impl TryFrom<sam_message::DeviceList> for DeviceList {
    type Error = ProtocolError;

    fn try_from(value: sam_message::DeviceList) -> Result<Self, Self::Error> {
        Ok(Self {
            _account_id: value
                .account_id
                .try_into()
                .map_err(|_| ProtocolError::MalformedServerMessage)?,
            _device_ids: value.device_ids.into_iter().map(|id| id.into()).collect(),
        })
    }
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
