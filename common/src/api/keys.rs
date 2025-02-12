use libsignal_protocol::{
    GenericSignedPreKey, IdentityKey, KyberPreKeyRecord, PreKeyRecord, SignedPreKeyRecord,
};
use serde::{Deserialize, Serialize};
use serde_with::{base64::Base64, serde_as};

macro_rules! define_key {
    ($name:ident) => {
        #[serde_as]
        #[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
        #[serde(rename_all = "camelCase")]
        pub struct $name {
            pub key_id: u32,
            #[serde_as(as = "Base64")]
            pub public_key: Box<[u8]>,
        }

        impl Key for $name {
            fn id(&self) -> u32 {
                return self.key_id;
            }
            fn public_key(&self) -> &[u8] {
                return &self.public_key;
            }
        }
    };
}

macro_rules! define_signed_key {
    ($name:ident) => {
        #[serde_as]
        #[derive(Debug, Serialize, Deserialize, Clone, Hash, PartialEq, Eq)]
        #[serde(rename_all = "camelCase")]
        pub struct $name {
            pub key_id: u32,
            #[serde_as(as = "Base64")]
            pub public_key: Box<[u8]>,
            #[serde_as(as = "Base64")]
            pub signature: Box<[u8]>,
        }

        impl Key for $name {
            fn id(&self) -> u32 {
                return self.key_id;
            }
            fn public_key(&self) -> &[u8] {
                return &self.public_key;
            }
        }

        impl SignedKey for $name {
            fn signature(&self) -> &[u8] {
                return &self.signature;
            }
        }
    };
}

pub trait Key {
    fn id(&self) -> u32;
    fn public_key(&self) -> &[u8];
}

pub trait SignedKey: Key {
    fn signature(&self) -> &[u8];
}

define_key!(EllipticCurvePreKey);

impl From<PreKeyRecord> for EllipticCurvePreKey {
    fn from(value: PreKeyRecord) -> Self {
        Self {
            key_id: value.id().expect("Can get ID").into(),
            public_key: value.public_key().expect("Can get public_key").serialize(),
        }
    }
}

define_signed_key!(SignedPreKey);

impl From<SignedPreKeyRecord> for SignedPreKey {
    fn from(value: SignedPreKeyRecord) -> Self {
        Self {
            key_id: value.id().expect("Can get ID").into(),
            public_key: value.public_key().expect("Can get public_key").serialize(),
            signature: value.signature().expect("Can get signature").into(),
        }
    }
}

define_signed_key!(PostQuantumPreKey);

impl From<KyberPreKeyRecord> for PostQuantumPreKey {
    fn from(value: KyberPreKeyRecord) -> Self {
        Self {
            key_id: value.id().expect("Can get ID").into(),
            public_key: value.public_key().expect("Can get public_key").serialize(),
            signature: value.signature().expect("Can get signature").into(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyBundleResponse {
    #[serde(with = "id_key")]
    pub identity_key: IdentityKey,
    pub bundles: Vec<KeyBundle>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyBundle {
    pub device_id: u32,
    pub registration_id: u32,
    pub pre_key: Option<EllipticCurvePreKey>,
    pub pq_pre_key: PostQuantumPreKey,
    pub signed_pre_key: SignedPreKey,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PublishKeyBundleRequest {
    pub pre_keys: Option<Vec<EllipticCurvePreKey>>,
    pub signed_pre_key: Option<SignedPreKey>,
    pub pq_pre_keys: Option<Vec<PostQuantumPreKey>>,
    pub pq_last_resort_pre_key: Option<PostQuantumPreKey>,
}

pub type PublishKeyBundle = PublishKeyBundleRequest;

pub mod id_key {
    use base64::{prelude::BASE64_STANDARD, Engine as _};
    use libsignal_protocol::IdentityKey;
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(key: &IdentityKey, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Convert IdentityKey to bytes and serialize them

        serializer.serialize_str(&BASE64_STANDARD.encode(key.serialize()))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<IdentityKey, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;
        let bytes = BASE64_STANDARD
            .decode(String::deserialize(deserializer)?)
            .map_err(|err| Error::custom(format!("{err}")))?;

        IdentityKey::decode(&bytes)
            .map_err(|e| Error::custom(format!("Failed to decode IdentityKey: {}", e)))
    }
}
