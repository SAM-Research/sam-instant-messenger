use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::RegistrationId;

use super::keys::PublishKeyBundle;

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
    pub account_id: Uuid,
    pub device_id: u32,
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
    pub key_bundle: PublishKeyBundle,
}
