use crate::auth::error::AuthorizationError;
use crate::ServerError;
use libsignal_protocol::IdentityKey;
use sam_common::api::keys::SignedKey;

pub fn verify_key<T: SignedKey>(identity: &IdentityKey, key: &T) -> Result<(), ServerError> {
    if !identity
        .public_key()
        .verify_signature(key.public_key(), key.signature())
    {
        Err(AuthorizationError::KeyVerification)?
    } else {
        Ok(())
    }
}
