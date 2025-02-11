use libsignal_protocol::IdentityKey;
use sam_common::api::keys::SignedPreKey;

use crate::ServerError;

pub fn verify_key(identity: &IdentityKey, key: &SignedPreKey) -> Result<(), ServerError> {
    if !identity
        .public_key()
        .verify_signature(&key.public_key, &key.signature)
    {
        Err(ServerError::KeyVerification)
    } else {
        Ok(())
    }
}
