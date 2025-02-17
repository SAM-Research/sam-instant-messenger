use crate::auth::password::Password;

#[derive(Clone, bon::Builder, PartialEq, Eq)]
pub struct Device {
    registration_id: u32,
    id: u32,
    name: String,
    creation: u128,
    password: Password,
}

impl Device {
    pub fn registration_id(&self) -> u32 {
        self.registration_id
    }

    pub fn id(&self) -> u32 {
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
