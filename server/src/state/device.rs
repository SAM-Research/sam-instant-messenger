use libsignal_protocol::DeviceId;

#[derive(bon::Builder, Clone)]
pub struct Device {
    pub registration_id: u32,
    pub device_id: DeviceId,
    pub name: String,
    pub created: u128,
    pub auth_token: String,
    pub salt: String,
}
