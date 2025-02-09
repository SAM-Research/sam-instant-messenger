use serde::{Deserialize, Serialize};

use super::keys::PublishKeyBundle;

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LinkDeviceToken {
    pub verification_code: String,
    pub token_identifier: String,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LinkDeviceResponse {
    pub aci: String,
    pub device_id: u32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkDeviceRequest {
    pub verification_code: String,
    pub device_activation: DeviceActivationInfo,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceActivationInfo {
    pub device_name: String,
    pub registration_id: u32,
    pub key_bundle: PublishKeyBundle,
}
