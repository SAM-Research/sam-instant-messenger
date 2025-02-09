use libsignal_protocol::{Aci, ProtocolAddress};

use crate::{
    state::{account::Account, device::Device},
    ServerError,
};

#[async_trait::async_trait]
pub trait AccountManager: Send {
    fn verify_link_token(&self, code: String) -> bool;
    fn get_link_token(&self, aci: Aci) -> String;

    async fn get_account(&self, aci: &Aci) -> Result<Account, ServerError>;
    async fn add_account(&mut self, account: Account) -> Result<(), ServerError>;
    async fn remove_account(&mut self, aci: Aci) -> Result<(), ServerError>;

    async fn add_device(&mut self, aci: &Aci, device: Device) -> Result<(), ServerError>;
    async fn get_device(&self, addr: ProtocolAddress) -> Result<Device, ServerError>;
    async fn get_devices(&self, aci: &Aci) -> Result<Vec<Device>, ServerError>;
    async fn remove_device(&mut self, addr: ProtocolAddress) -> Result<(), ServerError>;
    async fn add_used_device_link_token(&mut self, code: String) -> Result<(), ServerError>;
}
