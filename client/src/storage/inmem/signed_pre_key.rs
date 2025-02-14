use crate::storage::ProvidesKeyId;
use crate::ClientError;
use async_trait::async_trait;
use libsignal_protocol::{InMemSignedPreKeyStore, SignedPreKeyId};

#[async_trait(?Send)]
impl ProvidesKeyId<SignedPreKeyId> for InMemSignedPreKeyStore {
    async fn next_key_id(&self) -> Result<SignedPreKeyId, ClientError> {
        todo!()
    }
}
