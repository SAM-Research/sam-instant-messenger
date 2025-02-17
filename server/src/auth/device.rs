use std::{str::FromStr, time::Duration};

use base64::{
    prelude::{BASE64_STANDARD_NO_PAD, BASE64_URL_SAFE},
    Engine,
};
use hkdf::hmac::{Hmac, Mac};
use sam_common::{address::AccountId, api::device::LinkDeviceToken, time_now_millis};
use sha2::{Digest, Sha256};

use crate::ServerError;

pub fn create_token(secret: &str, account_id: AccountId) -> LinkDeviceToken {
    let claims = encode_claims(account_id);
    let signature = create_signature(secret, &claims);

    let token = format!("{}:{}", claims, BASE64_URL_SAFE.encode(signature));
    let id = create_id(&token);
    LinkDeviceToken::new(id, token)
}

pub fn verify_token(secret: &str, token: LinkDeviceToken) -> Result<AccountId, ServerError> {
    let (claims, b64_signature) = token
        .token()
        .split_once(":")
        .ok_or(ServerError::DeviceTokenMalformed)?;

    let expected_signature = create_signature(secret, claims);
    let signature = BASE64_URL_SAFE
        .decode(b64_signature)
        .map_err(|_| ServerError::DeviceSignatureDecodeError)?;

    if signature != expected_signature {
        return Err(ServerError::DeviceWrongSignature);
    }

    let (account_id, timestamp) = decode_claims(claims)?;
    let account_id =
        AccountId::from_str(account_id).map_err(|_| ServerError::DeviceTokenMalformed)?;

    let time_then = Duration::from_millis(
        timestamp
            .parse()
            .map_err(|_| ServerError::DeviceTokenMalformed)?,
    );
    let time_now = Duration::from_millis(time_now_millis() as u64);
    let elapsed = time_now - time_then;
    if elapsed.as_secs() > 600 {
        return Err(ServerError::DeviceLinkTooSlow);
    }
    Ok(account_id)
}

fn encode_claims(account_id: AccountId) -> String {
    format!("{}.{}", account_id, time_now_millis())
}

fn decode_claims(claims: &str) -> Result<(&str, &str), ServerError> {
    claims
        .split_once(".")
        .ok_or(ServerError::DeviceTokenMalformed)
}

fn create_signature(secret: &str, claims: &str) -> Vec<u8> {
    let mut mac =
        Hmac::<Sha256>::new_from_slice(secret.as_bytes()).expect("HMAC can take key of any size");
    mac.update(claims.as_bytes());
    mac.finalize().into_bytes().to_vec()
}

fn create_id(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    let digest = hasher.finalize();
    BASE64_STANDARD_NO_PAD.encode(digest)
}
