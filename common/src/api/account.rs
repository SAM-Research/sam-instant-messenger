use libsignal_protocol::IdentityKey;
use serde::{Deserialize, Serialize};

use crate::address::AccountId;

use super::{device::DeviceActivationInfo, keys::id_key};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistrationRequest {
    #[serde(with = "id_key")]
    pub identity_key: IdentityKey,
    pub device_activation: DeviceActivationInfo,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RegistrationResponse {
    pub account_id: AccountId,
}
