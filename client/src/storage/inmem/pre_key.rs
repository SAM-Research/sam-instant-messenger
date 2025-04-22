use async_trait::async_trait;
use libsignal_protocol::{InMemPreKeyStore, PreKeyId};
use log::debug;

use crate::storage::{error::KeyStoreError, ProvidesKeyId};

#[async_trait(?Send)]
impl ProvidesKeyId<PreKeyId> for InMemPreKeyStore {
    async fn next_key_id(&self) -> Result<PreKeyId, KeyStoreError> {
        let max: u32 = self
            .all_pre_key_ids()
            .max()
            .cloned()
            .map(|id| id.into())
            .unwrap_or_default();
        let next = max + 1;
        debug!("Next Pre Key ID: {next}");
        Ok(next.into())
    }
}
