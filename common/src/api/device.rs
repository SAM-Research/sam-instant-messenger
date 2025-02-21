use serde::{Deserialize, Serialize};

use crate::address::{AccountId, DeviceId, RegistrationId};

use super::keys::RegistrationPreKeys;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LinkDeviceToken {
    id: String,
    token: String,
}

impl LinkDeviceToken {
    pub fn new(id: String, token: String) -> Self {
        Self { id, token }
    }
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn token(&self) -> &str {
        &self.token
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LinkDeviceResponse {
    pub account_id: AccountId,
    pub device_id: DeviceId,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkDeviceRequest {
    pub token: LinkDeviceToken,
    pub device_activation: DeviceActivationInfo,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceActivationInfo {
    pub name: String,
    pub registration_id: RegistrationId,
    pub key_bundle: RegistrationPreKeys,
}
