use async_trait::async_trait;
use libsignal_protocol::{InMemKyberPreKeyStore, KyberPreKeyId};
use log::debug;

use crate::storage::{error::KeyStoreError, ProvidesKeyId};

#[async_trait(?Send)]
impl ProvidesKeyId<KyberPreKeyId> for InMemKyberPreKeyStore {
    async fn next_key_id(&self) -> Result<KyberPreKeyId, KeyStoreError> {
        let max: u32 = self
            .all_kyber_pre_key_ids()
            .max()
            .cloned()
            .map(|id| id.into())
            .unwrap_or_default();
        let next = max + 1;
        debug!("Next Kyber Pre Key ID: {next}");
        Ok(next.into())
    }
}
