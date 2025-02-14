use std::collections::HashMap;

use libsignal_protocol::IdentityKey;
use sam_common::api::keys::{Key, PostQuantumPreKey, PreKey, SignedPreKey};
use uuid::Uuid;

use crate::{
    auth::keys::verify_key,
    managers::traits::key_manager::{
        LastResortKeyManager, PqPreKeyManager, PreKeyManager, SignedPreKeyManager,
    },
    ServerError,
};

use super::device_key;

pub struct InMemoryKeyManager {
    pre_keys: HashMap<String, Vec<PreKey>>,
    signed_pre_keys: HashMap<String, SignedPreKey>,
    pq_pre_keys: HashMap<String, Vec<PostQuantumPreKey>>,
    last_resort_keys: HashMap<String, PostQuantumPreKey>,
}

#[async_trait::async_trait]
impl PreKeyManager for InMemoryKeyManager {
    async fn get_pre_key(
        &self,
        account_id: &Uuid,
        device_id: &u32,
    ) -> Result<Option<PreKey>, ServerError> {
        let key = device_key(account_id, *device_id);

        Ok(self
            .pre_keys
            .get(&key)
            .and_then(|keys| keys.first())
            .cloned())
    }

    async fn get_pre_keys(
        &self,
        account_id: &Uuid,
        device_id: &u32,
    ) -> Result<Vec<u32>, ServerError> {
        let key = device_key(account_id, *device_id);

        self.pre_keys
            .get(&key)
            .map(|keys| keys.iter().map(|k| k.id()).collect::<Vec<u32>>())
            .ok_or(ServerError::AccountNotExist)
    }

    async fn add_pre_key(
        &mut self,
        account_id: &Uuid,
        device_id: &u32,
        key: PreKey,
    ) -> Result<(), ServerError> {
        let dkey = device_key(account_id, *device_id);

        self.pre_keys
            .get_mut(&dkey)
            .map(|keys| keys.push(key))
            .ok_or(ServerError::AccountNotExist)
    }

    async fn remove_pre_key(
        &mut self,
        account_id: &Uuid,
        device_id: &u32,
        id: u32,
    ) -> Result<(), ServerError> {
        let dkey = device_key(account_id, *device_id);

        self.pre_keys
            .get_mut(&dkey)
            .and_then(|keys| {
                keys.iter()
                    .position(|k| k.id() == id)
                    .map(|index| (keys, index))
                    .map(|(keys, index)| keys.remove(index))
            })
            .ok_or(ServerError::KeyNotExist)
            .map(|_| ())
    }
}

#[async_trait::async_trait]
impl SignedPreKeyManager for InMemoryKeyManager {
    async fn get_signed_pre_key(
        &self,
        account_id: &Uuid,
        device_id: &u32,
    ) -> Result<SignedPreKey, ServerError> {
        let key = device_key(account_id, *device_id);

        self.signed_pre_keys
            .get(&key)
            .cloned()
            .ok_or(ServerError::AccountNotExist)
    }

    async fn set_signed_pre_key(
        &mut self,
        account_id: &Uuid,
        device_id: &u32,
        identity: &IdentityKey,
        key: SignedPreKey,
    ) -> Result<(), ServerError> {
        let dkey = device_key(account_id, *device_id);

        verify_key(identity, &key)?;

        let _ = self.signed_pre_keys.insert(dkey, key);
        Ok(())
    }

    async fn remove_signed_pre_key(
        &mut self,
        account_id: &Uuid,
        device_id: &u32,
    ) -> Result<(), ServerError> {
        let key = device_key(account_id, *device_id);

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
        account_id: &Uuid,
        device_id: &u32,
    ) -> Result<Option<PostQuantumPreKey>, ServerError> {
        let key = device_key(account_id, *device_id);

        Ok(self
            .pq_pre_keys
            .get(&key)
            .and_then(|keys| keys.first())
            .cloned())
    }

    async fn get_pq_pre_keys(
        &self,
        account_id: &Uuid,
        device_id: &u32,
    ) -> Result<Vec<u32>, ServerError> {
        let key = device_key(account_id, *device_id);

        self.pq_pre_keys
            .get(&key)
            .map(|keys| keys.iter().map(|k| k.id()).collect::<Vec<u32>>())
            .ok_or(ServerError::AccountNotExist)
    }

    async fn add_pq_pre_key(
        &mut self,
        account_id: &Uuid,
        device_id: &u32,
        identity: &IdentityKey,
        key: PostQuantumPreKey,
    ) -> Result<(), ServerError> {
        let dkey = device_key(account_id, *device_id);

        verify_key(identity, &key)?;

        self.pq_pre_keys
            .get_mut(&dkey)
            .map(|keys| keys.push(key))
            .ok_or(ServerError::AccountNotExist)
    }

    async fn remove_pq_pre_key(
        &mut self,
        account_id: &Uuid,
        device_id: &u32,
        id: u32,
    ) -> Result<(), ServerError> {
        let dkey = device_key(account_id, *device_id);

        self.pq_pre_keys
            .get_mut(&dkey)
            .and_then(|keys| {
                keys.iter()
                    .position(|k| k.id() == id)
                    .map(|index| (keys, index))
                    .map(|(keys, index)| keys.remove(index))
            })
            .ok_or(ServerError::KeyNotExist)
            .map(|_| ())
    }
}

#[async_trait::async_trait]
impl LastResortKeyManager for InMemoryKeyManager {
    async fn get_last_resort_key(
        &self,
        account_id: &Uuid,
        device_id: &u32,
    ) -> Result<PostQuantumPreKey, ServerError> {
        let key = device_key(account_id, *device_id);

        self.last_resort_keys
            .get(&key)
            .cloned()
            .ok_or(ServerError::KeyNotExist)
    }
    async fn set_last_resort_key(
        &mut self,
        account_id: &Uuid,
        device_id: &u32,
        identity: &IdentityKey,
        key: PostQuantumPreKey,
    ) -> Result<(), ServerError> {
        let dkey = device_key(account_id, *device_id);

        verify_key(identity, &key)?;

        let _ = self.last_resort_keys.insert(dkey, key);
        Ok(())
    }
    async fn remove_last_resort_key(
        &mut self,
        account_id: &Uuid,
        device_id: &u32,
    ) -> Result<(), ServerError> {
        let key = device_key(account_id, *device_id);

        self.signed_pre_keys
            .remove(&key)
            .ok_or(ServerError::KeyNotExist)
            .map(|_| ())
    }
}
