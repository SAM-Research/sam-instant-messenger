use axum::http::{HeaderMap, StatusCode};
use libsignal_protocol::{Aci, IdentityKey, ServiceId};
use rand::{rngs::OsRng, RngCore};
use sam_common::api::{authorization::BasicAuthorizationHeader, keys::UploadSignedPreKey};

use crate::{error::HTTPError, ServerError};

const SALT_SIZE: usize = 16;
const AUTH_TOKEN_HKDF_INFO: &[u8] = "authtoken".as_bytes();

pub fn auth_header(headers: &HeaderMap) -> Result<BasicAuthorizationHeader, HTTPError> {
    Ok(headers
        .get("Authorization")
        .ok_or_else(|| HTTPError {
            status_code: StatusCode::UNAUTHORIZED,
            body: "Missing authorization header".to_owned(),
        })?
        .to_str()
        .map_err(|err| HTTPError {
            status_code: StatusCode::UNAUTHORIZED,
            body: format!(
                "Authorization header could not be parsed as string: {}",
                err
            ),
        })?
        .parse()
        .map_err(|err| HTTPError {
            status_code: StatusCode::UNAUTHORIZED,
            body: format!("Authorization header could not be parsed: {}", err),
        })?)
}

pub fn service_id_aci(service_id: ServiceId) -> Result<Aci, HTTPError> {
    match service_id {
        ServiceId::Aci(aci) => Ok(aci),
        _ => Err(HTTPError {
            status_code: StatusCode::NOT_IMPLEMENTED,
            body: "SAM Server does not support PNI".into(),
        }),
    }
}

pub struct SaltedTokenHash {
    hash: String,
    salt: String,
}
impl SaltedTokenHash {
    pub fn new(hash: String, salt: String) -> Self {
        Self { hash, salt }
    }

    pub fn generate_for(credentials: &str) -> Result<Self, HTTPError> {
        fn generate_salt() -> String {
            let mut salt = [0u8; SALT_SIZE];
            OsRng.fill_bytes(&mut salt);
            hex::encode(salt)
        }

        let salt = generate_salt();
        let token = SaltedTokenHash::calculate(&salt, credentials)?;

        Ok(Self { salt, hash: token })
    }

    pub fn verify(&self, credentials: &str) -> Result<bool, HTTPError> {
        let their_value = SaltedTokenHash::calculate(&self.salt, credentials)?;
        Ok(self.hash == their_value)
    }

    fn calculate(salt: &str, token: &str) -> Result<String, HTTPError> {
        Ok(hex::encode(HKDF_DeriveSecrets(
            32,
            token.as_bytes(),
            Some(AUTH_TOKEN_HKDF_INFO),
            Some(salt.as_bytes()),
        )?))
    }

    pub fn hash(&self) -> String {
        self.hash.clone()
    }
    pub fn salt(&self) -> String {
        self.salt.clone()
    }
}

// Function taken from libsignal-bridge
#[allow(nonstandard_style)]
fn HKDF_DeriveSecrets(
    output_length: u32,
    ikm: &[u8],
    label: Option<&[u8]>,
    salt: Option<&[u8]>,
) -> Result<Vec<u8>, HTTPError> {
    let label = label.unwrap_or(&[]);
    let mut buffer = vec![0; output_length as usize];
    hkdf::Hkdf::<sha2::Sha256>::new(salt, ikm)
        .expand(label, &mut buffer)
        .map_err(|_| HTTPError {
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
            body: format!("output too long ({})", output_length),
        })?;
    Ok(buffer)
}

pub fn verify_key_signature(
    identity_key: &IdentityKey,
    key: &UploadSignedPreKey,
) -> Result<(), ServerError> {
    if !identity_key
        .public_key()
        .verify_signature(&key.public_key, &key.signature)
    {
        Err(ServerError::KeyVerification)
    } else {
        Ok(())
    }
}
