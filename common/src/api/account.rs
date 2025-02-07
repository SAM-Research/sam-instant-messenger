use libsignal_protocol::{GenericSignedPreKey, IdentityKey, KyberPreKeyRecord, SignedPreKeyRecord};
use serde::{Deserialize, Serialize};
use serde_with::{base64::Base64, serde_as};
use uuid::Uuid;

use super::keys::{id_key, UploadSignedPreKey};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistrationRequest {
    pub name: String,
    pub registration_id: u32,
    #[serde(with = "id_key")]
    pub identity_key: IdentityKey,
    pub signed_prekey: UploadSignedPreKey,
    pub post_quantum_prekey: UploadSignedPreKey,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RegistrationResponse {
    pub account_id: Uuid,
}
