use crate::storage::{error::KeyStoreError, ProvidesKeyId};
use async_trait::async_trait;
use libsignal_protocol::{InMemSignedPreKeyStore, SignedPreKeyId};
use log::debug;

#[async_trait(?Send)]
impl ProvidesKeyId<SignedPreKeyId> for InMemSignedPreKeyStore {
    async fn next_key_id(&self) -> Result<SignedPreKeyId, KeyStoreError> {
        let max: u32 = self
            .all_signed_pre_key_ids()
            .max()
            .cloned()
            .map(|id| id.into())
            .unwrap_or_default();
        let next = max + 1;
        debug!("Next Signed Pre Key ID: {next}");
        Ok(next.into())
    }
}
