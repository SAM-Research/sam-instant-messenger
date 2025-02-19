use async_trait::async_trait;
use libsignal_protocol::{InMemPreKeyStore, PreKeyId};

use crate::{storage::ProvidesKeyId, ClientError};

#[async_trait(?Send)]
impl ProvidesKeyId<PreKeyId> for InMemPreKeyStore {
    async fn next_key_id(&self) -> Result<PreKeyId, ClientError> {
        let max: u32 = self
            .all_pre_key_ids()
            .max()
            .cloned()
            .map(|id| id.into())
            .unwrap_or_default();
        Ok((max + 1).into())
    }
}
