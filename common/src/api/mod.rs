pub mod account;
pub mod device;
pub mod keys;

pub use account::{RegistrationRequest, RegistrationResponse};

use base64::{prelude::BASE64_STANDARD, Engine};
pub use device::{LinkDeviceRequest, LinkDeviceResponse, LinkDeviceToken};

pub use keys::{
    Decode, DecodeError, EcPreKey, Encode, EncodeError, Key, PqPreKey, PreKeyBundle,
    PublishPreKeys, SignedEcPreKey, SignedKey,
};

use crate::address::{AccountId, DeviceId};

pub fn registration_auth(username: String, password: String) -> String {
    format!(
        "Basic {}",
        BASE64_STANDARD.encode(format!("{}:{}", username, password))
    )
}

pub fn authorization(account_id: AccountId, device_id: DeviceId, password: String) -> String {
    format!(
        "Basic {}",
        BASE64_STANDARD.encode(format!("{account_id}.{device_id}:{password}"))
    )
}
