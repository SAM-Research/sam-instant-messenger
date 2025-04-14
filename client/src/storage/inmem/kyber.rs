use async_trait::async_trait;
use libsignal_protocol::{InMemKyberPreKeyStore, KyberPreKeyId};

use crate::storage::{error::StoreError, ProvidesKeyId};

#[async_trait(?Send)]
impl ProvidesKeyId<KyberPreKeyId> for InMemKyberPreKeyStore {
    async fn next_key_id(&self) -> Result<KyberPreKeyId, StoreError> {
        let max: u32 = self
            .all_kyber_pre_key_ids()
            .max()
            .cloned()
            .map(|id| id.into())
            .unwrap_or_default();
        Ok((max + 1).into())
    }
}
