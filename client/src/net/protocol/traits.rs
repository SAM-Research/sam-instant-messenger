use super::error::ProtocolError;
use async_trait::async_trait;
use log::debug;
use sam_common::{
    sam_message::{ClientEnvelope, DeviceList as ProtoDeviceList, ServerEnvelope},
    AccountId, DeviceId,
};
use tokio::sync::mpsc::Receiver;

pub struct DeviceList {
    pub account_id: AccountId,
    pub devices: Vec<DeviceId>,
}

impl TryFrom<ProtoDeviceList> for DeviceList {
    type Error = ProtocolError;

    fn try_from(value: ProtoDeviceList) -> Result<Self, Self::Error> {
        Ok(Self {
            account_id: AccountId::try_from(value.account_id)
                .inspect_err(|e| debug!("{e}"))
                .map_err(|_| ProtocolError::MalformedServerMessage)?,
            devices: value.device_ids.iter().map(|id| (*id).into()).collect(),
        })
    }
}

pub enum MessageStatus {
    ExtraDevices(Vec<DeviceList>),
    MissingDevices(Vec<DeviceList>),
    Ok,
}

#[async_trait::async_trait]
pub trait SamProtocolClient {
    async fn connect(&mut self) -> Result<Receiver<ServerEnvelope>, ProtocolError>;
    async fn disconnect(&mut self) -> Result<(), ProtocolError>;
    async fn is_connected(&self) -> bool;
    async fn send_message(
        &mut self,
        message: ClientEnvelope,
    ) -> Result<MessageStatus, ProtocolError>;
}

#[async_trait(?Send)]
pub trait ProtocolConfig {
    type ProtocolClient: SamProtocolClient;

    async fn create(
        self,
        account_id: AccountId,
        device_id: DeviceId,
        password: String,
    ) -> Result<Self::ProtocolClient, ProtocolError>;
}
