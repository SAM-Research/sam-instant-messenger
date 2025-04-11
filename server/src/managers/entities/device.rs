use rand::rngs::OsRng;
use sam_common::{
    address::{DeviceId, RegistrationId},
    AccountId,
};

use crate::auth::password::Password;

#[derive(Clone, bon::Builder, PartialEq, Eq)]
pub struct Device {
    registration_id: RegistrationId,
    id: DeviceId,
    name: String,
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

    pub fn password(&self) -> &Password {
        &self.password
    }
}
