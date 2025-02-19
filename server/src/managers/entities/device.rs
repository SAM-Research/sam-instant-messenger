use sam_common::address::{DeviceId, RegistrationId};

use crate::auth::password::Password;

#[derive(Clone, bon::Builder, PartialEq, Eq)]
pub struct Device {
    registration_id: RegistrationId,
    id: DeviceId,
    name: String,
    creation: u128,
    password: Password,
}

impl Device {
    pub fn registration_id(&self) -> RegistrationId {
        self.registration_id
    }

    pub fn id(&self) -> DeviceId {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn creation(&self) -> u128 {
        self.creation
    }

    pub fn password(&self) -> &Password {
        &self.password
    }
}
