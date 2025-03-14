use bon::Builder;
use sam_common::{AccountId, DeviceId};

#[derive(Debug, Clone, Builder)]
pub struct DecryptedEnvelope {
    content: Vec<u8>,
    source_account_id: AccountId,
    source_device_id: DeviceId,
}

impl DecryptedEnvelope {
    pub fn content_bytes(&self) -> &Vec<u8> {
        &self.content
    }
    pub fn content<T>(&self) -> Result<T, <T as TryFrom<Vec<u8>>>::Error>
    where
        T: TryFrom<Vec<u8>>,
    {
        T::try_from(self.content_bytes().clone())
    }

    pub fn source_account_id(&self) -> AccountId {
        self.source_account_id
    }

    pub fn source_device_id(&self) -> DeviceId {
        self.source_device_id
    }
}
