use libsignal_protocol::{
    GenericSignedPreKey, IdentityKey, KyberPreKeyRecord, PreKeyRecord, SignedPreKeyRecord,
};
use serde::{Deserialize, Serialize};
use serde_with::{base64::Base64, serde_as};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BundleResponse {
    #[serde(with = "id_key")]
    pub identity_key: IdentityKey,
    pub devices: Vec<KeyBundle>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyBundle {
    pub device_id: u32,
    pub registration_id: u32,
    pub pre_key: Option<UploadPreKey>,
    pub pq_pre_key: UploadSignedPreKey,
    pub signed_pre_key: UploadSignedPreKey,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PublishKeyBundleRequest {
    pub pre_keys: Option<Vec<UploadPreKey>>,
    pub signed_pre_key: Option<UploadSignedPreKey>,
    pub pq_pre_keys: Option<Vec<UploadSignedPreKey>>,
    pub pq_last_resort_pre_key: Option<UploadSignedPreKey>,
}

pub type PublishKeyBundle = PublishKeyBundleRequest;

#[serde_as]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UploadPreKey {
    pub key_id: u32,
    #[serde_as(as = "Base64")]
    pub public_key: Box<[u8]>,
}

impl From<PreKeyRecord> for UploadPreKey {
    fn from(value: PreKeyRecord) -> Self {
        UploadPreKey {
            key_id: value.id().expect("Can get ID").into(),
            public_key: value.public_key().expect("Can get public_key").serialize(),
        }
    }
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize, Clone, Hash, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UploadSignedPreKey {
    pub key_id: u32,
    #[serde_as(as = "Base64")]
    pub public_key: Box<[u8]>, // TODO: Make this a PublicKey and implement Serialize
    #[serde_as(as = "Base64")]
    pub signature: Box<[u8]>, // TODO: Make this a PublicKey and implement Serialize
}

impl From<SignedPreKeyRecord> for UploadSignedPreKey {
    fn from(value: SignedPreKeyRecord) -> Self {
        UploadSignedPreKey {
            key_id: value.id().expect("Can get ID").into(),
            public_key: value.public_key().expect("Can get public_key").serialize(),
            signature: value.signature().expect("Can get signature").into(),
        }
    }
}

impl From<KyberPreKeyRecord> for UploadSignedPreKey {
    fn from(value: KyberPreKeyRecord) -> Self {
        UploadSignedPreKey {
            key_id: value.id().expect("Can get ID").into(),
            public_key: value.public_key().expect("Can get public_key").serialize(),
            signature: value.signature().expect("Can get signature").into(),
        }
    }
}

pub mod id_key {
    use libsignal_protocol::IdentityKey;
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(key: &IdentityKey, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Convert IdentityKey to bytes and serialize them
        serializer.serialize_bytes(&key.serialize())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<IdentityKey, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;
        let bytes = Vec::<u8>::deserialize(deserializer)?;

        IdentityKey::decode(&bytes)
            .map_err(|e| Error::custom(format!("Failed to decode IdentityKey: {}", e)))
    }
}
