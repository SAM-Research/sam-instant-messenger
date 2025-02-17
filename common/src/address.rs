use std::{
    fmt::{self, Display, Formatter},
    str::FromStr,
};

use serde::{Deserialize, Serialize};

use derive_more::{Display, From, Into};
use rand::Rng;
use uuid::Uuid;

const REGISTRATION_ID_MAX: u32 = 16383;

macro_rules! define_id_type {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, From, Into, Serialize, Deserialize)]
        pub struct $name(Uuid);

        impl $name {
            pub fn generate() -> Self {
                Self(Uuid::new_v4())
            }
            pub fn parse_from_bytes(bytes: [u8; 16]) -> Self {
                Self(Uuid::from_bytes(bytes))
            }
            pub fn uuid(&self) -> &Uuid {
                &self.0
            }
        }

        impl FromStr for $name {
            type Err = uuid::Error;

            fn from_str(string: &str) -> Result<Self, Self::Err> {
                Ok(Self(Uuid::from_str(string)?))
            }
        }

        impl From<$name> for Vec<u8> {
            fn from(value: $name) -> Vec<u8> {
                value.uuid().into_bytes().to_vec()
            }
        }

        impl TryFrom<Vec<u8>> for $name {
            type Error = <[u8; 16] as TryFrom<Vec<u8>>>::Error;

            fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
                let bytes = value.try_into()?;
                Ok(Self::parse_from_bytes(bytes))
            }
        }

        impl Display for $name {
            fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0.to_string())
            }
        }
    };
}

define_id_type!(AccountId);
define_id_type!(MessageId);

#[derive(
    Copy,
    Clone,
    Debug,
    Hash,
    Eq,
    PartialEq,
    PartialOrd,
    Ord,
    From,
    Into,
    Default,
    Serialize,
    Deserialize,
)]
pub struct DeviceId(u32);

impl Display for DeviceId {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for DeviceId {
    type Err = <u32 as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.parse()?))
    }
}

#[derive(
    Copy,
    Clone,
    Debug,
    Hash,
    Eq,
    PartialEq,
    PartialOrd,
    Ord,
    From,
    Into,
    Default,
    Serialize,
    Deserialize,
)]
pub struct RegistrationId(u32);

impl RegistrationId {
    pub fn generate<R: Rng>(csprng: &mut R) -> Self {
        RegistrationId(csprng.gen_range(1..REGISTRATION_ID_MAX))
    }
}
impl Display for RegistrationId {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Display, Clone, Copy, Hash, PartialEq, Eq)]
#[display("{account_id}:  {device_id}")]
pub struct DeviceAddress {
    account_id: AccountId,
    device_id: DeviceId,
}

impl DeviceAddress {
    #[cfg(test)]
    pub fn random() -> Self {
        Self {
            account_id: Uuid::new_v4().into(),
            device_id: DeviceId::default(),
        }
    }
    pub fn new(account_id: AccountId, device_id: DeviceId) -> Self {
        Self {
            account_id,
            device_id,
        }
    }
    pub fn account_id(&self) -> AccountId {
        self.account_id.to_owned()
    }
    pub fn device_id(&self) -> DeviceId {
        self.device_id
    }
}
