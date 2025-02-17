use std::collections::HashMap;

use libsignal_protocol::IdentityKey;
use sam_common::{
    address::{AccountId, DeviceAddress, DeviceId},
    api::keys::{EcPreKey, Key, PqPreKey, SignedEcPreKey},
};

use crate::{
    auth::keys::verify_key,
    managers::traits::key_manager::{
        LastResortKeyManager, PqPreKeyManager, PreKeyManager, SignedPreKeyManager,
    },
    ServerError,
};

pub struct InMemoryKeyManager {
    pre_keys: HashMap<DeviceAddress, Vec<EcPreKey>>,
    signed_pre_keys: HashMap<DeviceAddress, SignedEcPreKey>,
    pq_pre_keys: HashMap<DeviceAddress, Vec<PqPreKey>>,
    last_resort_keys: HashMap<DeviceAddress, PqPreKey>,
}

impl Default for InMemoryKeyManager {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryKeyManager {
    pub fn new() -> Self {
        InMemoryKeyManager {
            pre_keys: HashMap::new(),
            signed_pre_keys: HashMap::new(),
            pq_pre_keys: HashMap::new(),
            last_resort_keys: HashMap::new(),
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

        if !self.pre_keys.contains_key(&dkey) {
            let _ = self.pre_keys.insert(dkey.clone(), Vec::new());
        }

        self.pre_keys
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
            .get_mut(&dkey)
            .and_then(|keys| {
                keys.iter()
                    .position(|k| k.id() == id)
                    .map(|index| (keys, index))
                    .map(|(keys, index)| keys.remove(index))
            })
            .ok_or(ServerError::KeyNotExist)?;

        if let Some(keys) = self.pre_keys.get(&dkey) {
            if keys.is_empty() {
                self.pre_keys.remove(&dkey);
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

        let _ = self.signed_pre_keys.insert(dkey, key);
        Ok(())
    }

    async fn remove_signed_pre_key(
        &mut self,
        account_id: AccountId,
        device_id: DeviceId,
    ) -> Result<(), ServerError> {
        let key = DeviceAddress::new(account_id, device_id);

        self.signed_pre_keys
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

        if !self.pq_pre_keys.contains_key(&dkey) {
            let _ = self.pq_pre_keys.insert(dkey.clone(), Vec::new());
        }

        self.pq_pre_keys
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
            .get_mut(&dkey)
            .and_then(|keys| {
                keys.iter()
                    .position(|k| k.id() == id)
                    .map(|index| (keys, index))
                    .map(|(keys, index)| keys.remove(index))
            })
            .ok_or(ServerError::KeyNotExist)?;

        if let Some(keys) = self.pq_pre_keys.get(&dkey) {
            if keys.is_empty() {
                self.pq_pre_keys.remove(&dkey);
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

        let _ = self.last_resort_keys.insert(dkey, key);
        Ok(())
    }
    async fn remove_last_resort_key(
        &mut self,
        account_id: AccountId,
        device_id: DeviceId,
    ) -> Result<(), ServerError> {
        let key = DeviceAddress::new(account_id, device_id);

        self.last_resort_keys
            .remove(&key)
            .ok_or(ServerError::KeyNotExist)
            .map(|_| ())
    }
}
