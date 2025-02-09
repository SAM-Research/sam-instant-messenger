use libsignal_protocol::{Aci, DeviceId, IdentityKey};

#[derive(Clone, bon::Builder)]
pub struct Account {
    pub aci: Aci,
    pub username: String,
    pub identity_key: IdentityKey,
    pub primary_device: DeviceId,
}
