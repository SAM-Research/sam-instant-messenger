use std::{
    collections::{hash_map::Entry, HashMap},
    sync::Arc,
};

use crate::{
    auth::keys::verify_key,
    managers::traits::key_manager::{
        LastResortKeyManager, PqPreKeyManager, PreKeyManager, SignedPreKeyManager,
    },
    ServerError,
};
use libsignal_protocol::IdentityKey;
use sam_common::{
    address::{AccountId, DeviceAddress, DeviceId},
    api::keys::{EcPreKey, Key, PqPreKey, SignedEcPreKey},
};
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct InMemoryKeyManager {
    pre_keys: Arc<Mutex<HashMap<DeviceAddress, Vec<EcPreKey>>>>,
    signed_pre_keys: Arc<Mutex<HashMap<DeviceAddress, SignedEcPreKey>>>,
    pq_pre_keys: Arc<Mutex<HashMap<DeviceAddress, Vec<PqPreKey>>>>,
    last_resort_keys: Arc<Mutex<HashMap<DeviceAddress, PqPreKey>>>,
}

impl Default for InMemoryKeyManager {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryKeyManager {
    pub fn new() -> Self {
        InMemoryKeyManager {
            pre_keys: Arc::new(Mutex::new(HashMap::new())),
            signed_pre_keys: Arc::new(Mutex::new(HashMap::new())),
            pq_pre_keys: Arc::new(Mutex::new(HashMap::new())),
            last_resort_keys: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait::async_trait]
impl PreKeyManager for InMemoryKeyManager {
    async fn get_pre_key(
        &self,
        account_id: AccountId,
        device_id: DeviceId,
    ) -> Result<Option<EcPreKey>, ServerError> {
        let key = DeviceAddress::new(account_id, device_id);

        Ok(self
            .pre_keys
            .lock()
            .await
            .get(&key)
            .and_then(|keys| keys.first())
            .cloned())
    }

    async fn get_pre_keys(
        &self,
        account_id: AccountId,
        device_id: DeviceId,
    ) -> Result<Vec<u32>, ServerError> {
        let key = DeviceAddress::new(account_id, device_id);

        self.pre_keys
            .lock()
            .await
            .get(&key)
            .map(|keys| keys.iter().map(|k| k.id()).collect::<Vec<u32>>())
            .ok_or(ServerError::AccountNotExist)
    }

    async fn get_pre_key_ids(
        &self,
        account_id: AccountId,
        device_id: DeviceId,
    ) -> Result<Vec<u32>, ServerError> {
        let key = DeviceAddress::new(account_id, device_id);

        self.pre_keys
            .lock()
            .await
            .get(&key)
            .map(|keys| keys.iter().map(|k| k.id()).collect::<Vec<u32>>())
            .ok_or(ServerError::AccountNotExist)
    }

    async fn add_pre_key(
        &mut self,
        account_id: AccountId,
        device_id: DeviceId,
        key: EcPreKey,
    ) -> Result<(), ServerError> {
        let dkey = DeviceAddress::new(account_id, device_id);

        if let Entry::Vacant(e) = self.pre_keys.lock().await.entry(dkey) {
            e.insert(Vec::new());
        }

        self.pre_keys
            .lock()
            .await
            .get_mut(&dkey)
            .map(|keys| keys.push(key))
            .ok_or(ServerError::AccountNotExist)
    }

    async fn remove_pre_key(
        &mut self,
        account_id: AccountId,
        device_id: DeviceId,
        id: u32,
    ) -> Result<(), ServerError> {
        let dkey = DeviceAddress::new(account_id, device_id);

        self.pre_keys
            .lock()
            .await
            .get_mut(&dkey)
            .and_then(|keys| {
                keys.iter()
                    .position(|k| k.id() == id)
                    .map(|index| (keys, index))
                    .map(|(keys, index)| keys.remove(index))
            })
            .ok_or(ServerError::KeyNotExist)?;
        {
            let mut pre_keys = self.pre_keys.lock().await;
            if let Some(keys) = pre_keys.get(&dkey) {
                if keys.is_empty() {
                    pre_keys.remove(&dkey);
                }
            }
        }
        Ok(())
    }
}

#[async_trait::async_trait]
impl SignedPreKeyManager for InMemoryKeyManager {
    async fn get_signed_pre_key(
        &self,
        account_id: AccountId,
        device_id: DeviceId,
    ) -> Result<SignedEcPreKey, ServerError> {
        let key = DeviceAddress::new(account_id, device_id);

        self.signed_pre_keys
            .lock()
            .await
            .get(&key)
            .cloned()
            .ok_or(ServerError::AccountNotExist)
    }

    async fn set_signed_pre_key(
        &mut self,
        account_id: AccountId,
        device_id: DeviceId,
        identity: &IdentityKey,
        key: SignedEcPreKey,
    ) -> Result<(), ServerError> {
        let dkey = DeviceAddress::new(account_id, device_id);

        verify_key(identity, &key)?;

        let _ = self.signed_pre_keys.lock().await.insert(dkey, key);
        Ok(())
    }

    async fn remove_signed_pre_key(
        &mut self,
        account_id: AccountId,
        device_id: DeviceId,
    ) -> Result<(), ServerError> {
        let key = DeviceAddress::new(account_id, device_id);

        self.signed_pre_keys
            .lock()
            .await
            .remove(&key)
            .ok_or(ServerError::KeyNotExist)
            .map(|_| ())
    }
}

#[async_trait::async_trait]
impl PqPreKeyManager for InMemoryKeyManager {
    async fn get_pq_pre_key(
        &self,
        account_id: AccountId,
        device_id: DeviceId,
    ) -> Result<Option<PqPreKey>, ServerError> {
        let key = DeviceAddress::new(account_id, device_id);

        Ok(self
            .pq_pre_keys
            .lock()
            .await
            .get(&key)
            .and_then(|keys| keys.first())
            .cloned())
    }

    async fn get_pq_pre_keys(
        &self,
        account_id: AccountId,
        device_id: DeviceId,
    ) -> Result<Vec<u32>, ServerError> {
        let key = DeviceAddress::new(account_id, device_id);

        self.pq_pre_keys
            .lock()
            .await
            .get(&key)
            .map(|keys| keys.iter().map(|k| k.id()).collect::<Vec<u32>>())
            .ok_or(ServerError::AccountNotExist)
    }

    async fn get_pq_pre_key_ids(
        &self,
        account_id: AccountId,
        device_id: DeviceId,
    ) -> Result<Vec<u32>, ServerError> {
        let key = DeviceAddress::new(account_id, device_id);

        self.pq_pre_keys
            .lock()
            .await
            .get(&key)
            .map(|keys| keys.iter().map(|k| k.id()).collect::<Vec<u32>>())
            .ok_or(ServerError::AccountNotExist)
    }

    async fn add_pq_pre_key(
        &mut self,
        account_id: AccountId,
        device_id: DeviceId,
        identity: &IdentityKey,
        key: PqPreKey,
    ) -> Result<(), ServerError> {
        let dkey = DeviceAddress::new(account_id, device_id);

        verify_key(identity, &key)?;

        if let Entry::Vacant(e) = self.pq_pre_keys.lock().await.entry(dkey) {
            e.insert(Vec::new());
        }

        self.pq_pre_keys
            .lock()
            .await
            .get_mut(&dkey)
            .map(|keys| keys.push(key))
            .ok_or(ServerError::AccountNotExist)
    }

    async fn remove_pq_pre_key(
        &mut self,
        account_id: AccountId,
        device_id: DeviceId,
        id: u32,
    ) -> Result<(), ServerError> {
        let dkey = DeviceAddress::new(account_id, device_id);

        self.pq_pre_keys
            .lock()
            .await
            .get_mut(&dkey)
            .and_then(|keys| {
                keys.iter()
                    .position(|k| k.id() == id)
                    .map(|index| (keys, index))
                    .map(|(keys, index)| keys.remove(index))
            })
            .ok_or(ServerError::KeyNotExist)?;
        {
            let mut pq_pre_keys = self.pq_pre_keys.lock().await;
            if let Some(keys) = pq_pre_keys.get(&dkey) {
                if keys.is_empty() {
                    pq_pre_keys.remove(&dkey);
                }
            }
        }
        Ok(())
    }
}

#[async_trait::async_trait]
impl LastResortKeyManager for InMemoryKeyManager {
    async fn get_last_resort_key(
        &self,
        account_id: AccountId,
        device_id: DeviceId,
    ) -> Result<PqPreKey, ServerError> {
        let key = DeviceAddress::new(account_id, device_id);

        self.last_resort_keys
            .lock()
            .await
            .get(&key)
            .cloned()
            .ok_or(ServerError::KeyNotExist)
    }
    async fn set_last_resort_key(
        &mut self,
        account_id: AccountId,
        device_id: DeviceId,
        identity: &IdentityKey,
        key: PqPreKey,
    ) -> Result<(), ServerError> {
        let dkey = DeviceAddress::new(account_id, device_id);

        verify_key(identity, &key)?;

        let _ = self.last_resort_keys.lock().await.insert(dkey, key);
        Ok(())
    }
    async fn remove_last_resort_key(
        &mut self,
        account_id: AccountId,
        device_id: DeviceId,
    ) -> Result<(), ServerError> {
        let key = DeviceAddress::new(account_id, device_id);

        self.last_resort_keys
            .lock()
            .await
            .remove(&key)
            .ok_or(ServerError::KeyNotExist)
            .map(|_| ())
    }
}
