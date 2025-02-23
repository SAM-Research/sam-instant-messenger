use libsignal_protocol::{
    GenericSignedPreKey, IdentityKey, KyberPreKeyRecord, PreKeyRecord, SignedPreKeyRecord,
};
use serde::{Deserialize, Serialize};
use serde_with::{base64::Base64, serde_as};

use crate::LibError;

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

pub trait Key: Sized + Send {
    fn id(&self) -> u32;
    fn public_key(&self) -> &[u8];
}

pub trait SignedKey: Key {
    fn signature(&self) -> &[u8];
}

define_key!(EcPreKey);

impl From<PreKeyRecord> for EcPreKey {
    fn from(value: PreKeyRecord) -> Self {
        Self {
            key_id: value.id().expect("Can get ID").into(),
            public_key: value.public_key().expect("Can get public_key").serialize(),
        }
    }
}

define_signed_key!(SignedEcPreKey);

impl From<SignedPreKeyRecord> for SignedEcPreKey {
    fn from(value: SignedPreKeyRecord) -> Self {
        Self {
            key_id: value.id().expect("Can get ID").into(),
            public_key: value.public_key().expect("Can get public_key").serialize(),
            signature: value.signature().expect("Can get signature").into(),
        }
    }
}

define_signed_key!(PqPreKey);

impl From<KyberPreKeyRecord> for PqPreKey {
    fn from(value: KyberPreKeyRecord) -> Self {
        Self {
            key_id: value.id().expect("Can get ID").into(),
            public_key: value.public_key().expect("Can get public_key").serialize(),
            signature: value.signature().expect("Can get signature").into(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PreKeyBundles {
    #[serde(with = "id_key")]
    pub identity_key: IdentityKey,
    pub bundles: Vec<PreKeyBundle>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PreKeyBundle {
    pub device_id: u32,
    pub registration_id: u32,
    pub pre_key: Option<EcPreKey>,
    pub pq_pre_key: PqPreKey,
    pub signed_pre_key: SignedEcPreKey,
}

impl PreKeyBundle {
    pub fn new(
        device_id: impl Into<u32>,
        registration_id: impl Into<u32>,
        pre_key: Option<EcPreKey>,
        pq_pre_key: PqPreKey,
        signed_pre_key: SignedEcPreKey,
    ) -> Self {
        Self {
            device_id: device_id.into(),
            registration_id: registration_id.into(),
            pre_key,
            pq_pre_key,
            signed_pre_key,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PublishPreKeys {
    pub pre_keys: Option<Vec<EcPreKey>>,
    pub signed_pre_key: Option<SignedEcPreKey>,
    pub pq_pre_keys: Option<Vec<PqPreKey>>,
    pub pq_last_resort_pre_key: Option<PqPreKey>,
}

impl From<RegistrationPreKeys> for PublishPreKeys {
    fn from(value: RegistrationPreKeys) -> Self {
        Self {
            pre_keys: value.pre_keys,
            signed_pre_key: Some(value.signed_pre_key),
            pq_pre_keys: value.pq_pre_keys,
            pq_last_resort_pre_key: Some(value.pq_last_resort_pre_key),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RegistrationPreKeys {
    pub pre_keys: Option<Vec<EcPreKey>>,
    pub signed_pre_key: SignedEcPreKey,
    pub pq_pre_keys: Option<Vec<PqPreKey>>,
    pub pq_last_resort_pre_key: PqPreKey,
}

impl TryFrom<PublishPreKeys> for RegistrationPreKeys {
    type Error = LibError;

    fn try_from(value: PublishPreKeys) -> Result<Self, Self::Error> {
        Ok(Self {
            pre_keys: value.pre_keys,
            signed_pre_key: value
                .signed_pre_key
                .ok_or(LibError::RegistrationKeyFieldsRequired)?,
            pq_pre_keys: value.pq_pre_keys,
            pq_last_resort_pre_key: value
                .pq_last_resort_pre_key
                .ok_or(LibError::RegistrationKeyFieldsRequired)?,
        })
    }
}

pub mod id_key {
    use base64::{prelude::BASE64_STANDARD, Engine as _};
    use libsignal_protocol::IdentityKey;
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(key: &IdentityKey, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
            .map_err(|err| Error::custom(format!("Failed to decode IdentityKey: {err}")))
    }
}
