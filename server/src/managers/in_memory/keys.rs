use std::{collections::HashMap, sync::Arc};

use libsignal_protocol::IdentityKey;
use sam_common::api::keys::{EcPreKey, Key, PqPreKey, SignedEcPreKey};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::{
    auth::keys::verify_key,
    managers::traits::key_manager::{
        LastResortKeyManager, PqPreKeyManager, PreKeyManager, SignedPreKeyManager,
    },
    ServerError,
};

use super::device_key;

#[derive(Clone)]
pub struct InMemoryKeyManager {
    pre_keys: Arc<Mutex<HashMap<String, Vec<EcPreKey>>>>,
    signed_pre_keys: Arc<Mutex<HashMap<String, SignedEcPreKey>>>,
    pq_pre_keys: Arc<Mutex<HashMap<String, Vec<PqPreKey>>>>,
    last_resort_keys: Arc<Mutex<HashMap<String, PqPreKey>>>,
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
        account_id: Uuid,
        device_id: u32,
    ) -> Result<Option<EcPreKey>, ServerError> {
        let key = device_key(account_id, device_id);

        Ok(self
            .pre_keys
            .lock()
            .await
            .get(&key)
            .and_then(|keys| keys.first())
            .cloned())
    }

    async fn get_pre_key_ids(
        &self,
        account_id: Uuid,
        device_id: u32,
    ) -> Result<Vec<u32>, ServerError> {
        let key = device_key(account_id, device_id);

        self.pre_keys
            .lock()
            .await
            .get(&key)
            .map(|keys| keys.iter().map(|k| k.id()).collect::<Vec<u32>>())
            .ok_or(ServerError::AccountNotExist)
    }

    async fn add_pre_key(
        &mut self,
        account_id: Uuid,
        device_id: u32,
        key: EcPreKey,
    ) -> Result<(), ServerError> {
        let dkey = device_key(account_id, device_id);

        if !self.pre_keys.lock().await.contains_key(&dkey) {
            let _ = self.pre_keys.lock().await.insert(dkey.clone(), Vec::new());
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
        account_id: Uuid,
        device_id: u32,
        id: u32,
    ) -> Result<(), ServerError> {
        let dkey = device_key(account_id, device_id);

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
        account_id: Uuid,
        device_id: u32,
    ) -> Result<SignedEcPreKey, ServerError> {
        let key = device_key(account_id, device_id);

        self.signed_pre_keys
            .lock()
            .await
            .get(&key)
            .cloned()
            .ok_or(ServerError::AccountNotExist)
    }

    async fn set_signed_pre_key(
        &mut self,
        account_id: Uuid,
        device_id: u32,
        identity: &IdentityKey,
        key: SignedEcPreKey,
    ) -> Result<(), ServerError> {
        let dkey = device_key(account_id, device_id);

        verify_key(identity, &key)?;

        let _ = self.signed_pre_keys.lock().await.insert(dkey, key);
        Ok(())
    }

    async fn remove_signed_pre_key(
        &mut self,
        account_id: Uuid,
        device_id: u32,
    ) -> Result<(), ServerError> {
        let key = device_key(account_id, device_id);

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
        account_id: Uuid,
        device_id: u32,
    ) -> Result<Option<PqPreKey>, ServerError> {
        let key = device_key(account_id, device_id);

        Ok(self
            .pq_pre_keys
            .lock()
            .await
            .get(&key)
            .and_then(|keys| keys.first())
            .cloned())
    }

    async fn get_pq_pre_key_ids(
        &self,
        account_id: Uuid,
        device_id: u32,
    ) -> Result<Vec<u32>, ServerError> {
        let key = device_key(account_id, device_id);

        self.pq_pre_keys
            .lock()
            .await
            .get(&key)
            .map(|keys| keys.iter().map(|k| k.id()).collect::<Vec<u32>>())
            .ok_or(ServerError::AccountNotExist)
    }

    async fn add_pq_pre_key(
        &mut self,
        account_id: Uuid,
        device_id: u32,
        identity: &IdentityKey,
        key: PqPreKey,
    ) -> Result<(), ServerError> {
        let dkey = device_key(account_id, device_id);

        verify_key(identity, &key)?;

        if !self.pq_pre_keys.lock().await.contains_key(&dkey) {
            let _ = self
                .pq_pre_keys
                .lock()
                .await
                .insert(dkey.clone(), Vec::new());
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
        account_id: Uuid,
        device_id: u32,
        id: u32,
    ) -> Result<(), ServerError> {
        let dkey = device_key(account_id, device_id);

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
        account_id: Uuid,
        device_id: u32,
    ) -> Result<PqPreKey, ServerError> {
        let key = device_key(account_id, device_id);

        self.last_resort_keys
            .lock()
            .await
            .get(&key)
            .cloned()
            .ok_or(ServerError::KeyNotExist)
    }
    async fn set_last_resort_key(
        &mut self,
        account_id: Uuid,
        device_id: u32,
        identity: &IdentityKey,
        key: PqPreKey,
    ) -> Result<(), ServerError> {
        let dkey = device_key(account_id, device_id);

        verify_key(identity, &key)?;

        let _ = self.last_resort_keys.lock().await.insert(dkey, key);
        Ok(())
    }
    async fn remove_last_resort_key(
        &mut self,
        account_id: Uuid,
        device_id: u32,
    ) -> Result<(), ServerError> {
        let key = device_key(account_id, device_id);

        self.last_resort_keys
            .lock()
            .await
            .remove(&key)
            .ok_or(ServerError::KeyNotExist)
            .map(|_| ())
    }
}
