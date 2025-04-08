use std::{
    collections::{hash_map::Entry, HashMap},
    sync::Arc,
};

use crate::managers::{error::KeyManagerError, KeyManagerType};
use crate::{
    auth::keys::verify_key,
    managers::traits::key_manager::{
        EcPreKeyManager, LastResortPqPreKeyManager, PqPreKeyManager, SignedPreKeyManager,
    },
};
use async_trait::async_trait;
use libsignal_protocol::IdentityKey;
use sam_common::{
    address::{AccountId, DeviceAddress, DeviceId},
    api::keys::{EcPreKey, Key, PqPreKey, SignedEcPreKey},
};
use tokio::sync::Mutex;

#[derive(Clone, Debug, Default)]
pub struct InMemoryEcPreKeyManager {
    pre_keys: Arc<Mutex<HashMap<DeviceAddress, Vec<EcPreKey>>>>,
}

#[derive(Clone, Debug, Default)]
pub struct InMemorySignedPreKeyManager {
    signed_pre_keys: Arc<Mutex<HashMap<DeviceAddress, SignedEcPreKey>>>,
}

#[derive(Clone, Debug, Default)]
pub struct InMemoryPqPreKeyManager {
    pq_pre_keys: Arc<Mutex<HashMap<DeviceAddress, Vec<PqPreKey>>>>,
}

#[derive(Clone, Debug, Default)]
pub struct InMemoryLastResortPqPreKeyManager {
    last_resort_keys: Arc<Mutex<HashMap<DeviceAddress, PqPreKey>>>,
}

#[derive(Clone)]
pub struct InMemoryKeyManager;

impl KeyManagerType for InMemoryKeyManager {
    type EcPreKeyManager = InMemoryEcPreKeyManager;

    type PqPreKeyManager = InMemoryPqPreKeyManager;

    type SignedPreKeyManager = InMemorySignedPreKeyManager;

    type LastResortPqPreKeyManager = InMemoryLastResortPqPreKeyManager;
}

#[async_trait]
impl EcPreKeyManager for InMemoryEcPreKeyManager {
    async fn get_pre_key(
        &self,
        account_id: AccountId,
        device_id: DeviceId,
    ) -> Result<Option<EcPreKey>, KeyManagerError> {
        let key = DeviceAddress::new(account_id, device_id);

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
        account_id: AccountId,
        device_id: DeviceId,
    ) -> Result<Option<Vec<u32>>, KeyManagerError> {
        let key = DeviceAddress::new(account_id, device_id);

        Ok(self
            .pre_keys
            .lock()
            .await
            .get(&key)
            .map(|keys| keys.iter().map(|k| k.id()).collect::<Vec<u32>>()))
    }

    async fn add_pre_key(
        &mut self,
        account_id: AccountId,
        device_id: DeviceId,
        key: EcPreKey,
    ) -> Result<(), KeyManagerError> {
        let dkey = DeviceAddress::new(account_id, device_id);

        if let Entry::Vacant(e) = self.pre_keys.lock().await.entry(dkey) {
            e.insert(Vec::new());
        }

        self.pre_keys
            .lock()
            .await
            .get_mut(&dkey)
            .map(|keys| keys.push(key))
            .ok_or(KeyManagerError::AccountDoesNotExist)
    }

    async fn remove_pre_key(
        &mut self,
        account_id: AccountId,
        device_id: DeviceId,
        id: u32,
    ) -> Result<(), KeyManagerError> {
        let dkey = DeviceAddress::new(account_id, device_id);

        self.pre_keys.lock().await.get_mut(&dkey).and_then(|keys| {
            keys.iter()
                .position(|k| k.id() == id)
                .map(|index| (keys, index))
                .map(|(keys, index)| keys.remove(index))
        });
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

#[async_trait]
impl SignedPreKeyManager for InMemorySignedPreKeyManager {
    async fn get_signed_pre_key(
        &self,
        account_id: AccountId,
        device_id: DeviceId,
    ) -> Result<SignedEcPreKey, KeyManagerError> {
        let key = DeviceAddress::new(account_id, device_id);

        self.signed_pre_keys
            .lock()
            .await
            .get(&key)
            .cloned()
            .ok_or(KeyManagerError::AccountDoesNotExist)
    }

    async fn set_signed_pre_key(
        &mut self,
        account_id: AccountId,
        device_id: DeviceId,
        identity: &IdentityKey,
        key: SignedEcPreKey,
    ) -> Result<(), KeyManagerError> {
        let dkey = DeviceAddress::new(account_id, device_id);

        verify_key(identity, &key)?;

        let _ = self.signed_pre_keys.lock().await.insert(dkey, key);
        Ok(())
    }

    async fn remove_signed_pre_key(
        &mut self,
        account_id: AccountId,
        device_id: DeviceId,
    ) -> Result<(), KeyManagerError> {
        let key = DeviceAddress::new(account_id, device_id);

        let _ = self
            .signed_pre_keys
            .lock()
            .await
            .remove(&key)
            .ok_or(KeyManagerError::AccountDoesNotExist)?;
        Ok(())
    }
}

#[async_trait]
impl PqPreKeyManager for InMemoryPqPreKeyManager {
    async fn get_pq_pre_key(
        &self,
        account_id: AccountId,
        device_id: DeviceId,
    ) -> Result<Option<PqPreKey>, KeyManagerError> {
        let key = DeviceAddress::new(account_id, device_id);

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
        account_id: AccountId,
        device_id: DeviceId,
    ) -> Result<Option<Vec<u32>>, KeyManagerError> {
        let key = DeviceAddress::new(account_id, device_id);

        Ok(self
            .pq_pre_keys
            .lock()
            .await
            .get(&key)
            .map(|keys| keys.iter().map(|k| k.id()).collect::<Vec<u32>>()))
    }

    async fn add_pq_pre_key(
        &mut self,
        account_id: AccountId,
        device_id: DeviceId,
        identity: &IdentityKey,
        key: PqPreKey,
    ) -> Result<(), KeyManagerError> {
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
            .ok_or(KeyManagerError::AccountDoesNotExist)?;
        Ok(())
    }

    async fn remove_pq_pre_key(
        &mut self,
        account_id: AccountId,
        device_id: DeviceId,
        id: u32,
    ) -> Result<(), KeyManagerError> {
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
            .ok_or(KeyManagerError::KeyDoesNotExist)?;
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

#[async_trait]
impl LastResortPqPreKeyManager for InMemoryLastResortPqPreKeyManager {
    async fn get_last_resort_key(
        &self,
        account_id: AccountId,
        device_id: DeviceId,
    ) -> Result<PqPreKey, KeyManagerError> {
        let key = DeviceAddress::new(account_id, device_id);

        self.last_resort_keys
            .lock()
            .await
            .get(&key)
            .cloned()
            .ok_or(KeyManagerError::KeyDoesNotExist)
    }

    async fn set_last_resort_key(
        &mut self,
        account_id: AccountId,
        device_id: DeviceId,
        identity: &IdentityKey,
        key: PqPreKey,
    ) -> Result<(), KeyManagerError> {
        let dkey = DeviceAddress::new(account_id, device_id);

        verify_key(identity, &key)?;

        let _ = self.last_resort_keys.lock().await.insert(dkey, key);
        Ok(())
    }

    async fn remove_last_resort_key(
        &mut self,
        account_id: AccountId,
        device_id: DeviceId,
    ) -> Result<(), KeyManagerError> {
        let key = DeviceAddress::new(account_id, device_id);

        self.last_resort_keys
            .lock()
            .await
            .remove(&key)
            .map(|_| ())
            .ok_or(KeyManagerError::KeyDoesNotExist)
    }
}
