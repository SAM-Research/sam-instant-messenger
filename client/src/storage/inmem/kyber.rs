use async_trait::async_trait;
use libsignal_protocol::{InMemKyberPreKeyStore, KyberPreKeyId};

use crate::{storage::ProvidesKeyId, ClientError};

#[async_trait(?Send)]
impl ProvidesKeyId<KyberPreKeyId> for InMemKyberPreKeyStore {
    async fn next_key_id(&self) -> Result<KyberPreKeyId, ClientError> {
        todo!()
    }
}
