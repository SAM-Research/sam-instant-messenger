use std::collections::HashMap;

use async_trait::async_trait;
use sam_common::{
    address::{DeviceAddress, RegistrationId},
    api::{EcPreKey, PqPreKey, PreKeyBundle, PublishPreKeys, SignedEcPreKey},
};

use crate::storage::{error::KeyStoreError, traits::KeyStore};

#[derive(Debug)]
pub struct InMemoryKeyStore {
    prekeys: HashMap<DeviceAddress, Vec<EcPreKey>>,
    last_resort_pq_prekeys: HashMap<DeviceAddress, PqPreKey>,
    pq_prekeys: HashMap<DeviceAddress, Vec<PqPreKey>>,
    signed_prekeys: HashMap<DeviceAddress, SignedEcPreKey>,
}

#[async_trait(?Send)]
impl KeyStore for InMemoryKeyStore {
    async fn store_signed_ec_pre_key(
        &mut self,
        spk: SignedEcPreKey,
        address: &DeviceAddress,
    ) -> Result<(), KeyStoreError> {
        self.signed_prekeys
            .insert(address.to_owned(), spk.to_owned());
        Ok(())
    }
    async fn store_last_resort_pq_pre_key(
        &mut self,
        pq_spk: PqPreKey,
        address: &DeviceAddress,
    ) -> Result<(), KeyStoreError> {
        self.last_resort_pq_prekeys
            .insert(address.to_owned(), pq_spk.to_owned());
        Ok(())
    }

    async fn store_pq_pre_keys(
        &mut self,
        otpks: Vec<PqPreKey>,
        owner: &DeviceAddress,
    ) -> Result<(), KeyStoreError> {
        self.pq_prekeys.insert(owner.to_owned(), otpks);
        Ok(())
    }
    async fn store_ec_pre_keys(
        &mut self,
        otpks: Vec<EcPreKey>,
        owner: &DeviceAddress,
    ) -> Result<(), KeyStoreError> {
        self.prekeys.insert(owner.to_owned(), otpks);
        Ok(())
    }

    async fn store_key_bundle(
        &mut self,
        data: PublishPreKeys,
        address: &DeviceAddress,
    ) -> Result<(), KeyStoreError> {
        if let Some(prekeys) = data.pre_keys {
            self.prekeys.insert(address.to_owned(), prekeys);
        }
        if let Some(prekeys) = data.pq_pre_keys {
            self.pq_prekeys.insert(address.to_owned(), prekeys);
        }
        if let Some(prekeys) = data.signed_pre_key {
            self.signed_prekeys.insert(address.to_owned(), prekeys);
        }
        if let Some(prekey) = data.pq_last_resort_pre_key {
            self.last_resort_pq_prekeys
                .insert(address.to_owned(), prekey);
        }
        Ok(())
    }

    async fn get_key_bundle(
        &mut self,
        registration_id: RegistrationId,
        address: &DeviceAddress,
    ) -> Result<PreKeyBundle, KeyStoreError> {
        let pre_key = self.prekeys.get_mut(address).and_then(|ecs| ecs.pop());
        let pq_pre_key = self
            .pq_prekeys
            .get_mut(address)
            .map(|pqs| pqs.pop())
            .unwrap_or_else(|| self.last_resort_pq_prekeys.get(address).cloned())
            .ok_or(KeyStoreError::AddressNotFound(address.to_owned()))?;
        let signed_pre_key = self
            .signed_prekeys
            .get_mut(address)
            .map(|spk| spk.clone())
            .ok_or(KeyStoreError::AddressNotFound(address.to_owned()))?;

        Ok(PreKeyBundle::new(
            address.device_id(),
            registration_id,
            pre_key,
            pq_pre_key,
            signed_pre_key,
        ))
    }
    async fn get_ec_pre_key_count(&self, address: &DeviceAddress) -> Result<usize, KeyStoreError> {
        Ok(self
            .prekeys
            .get(address)
            .ok_or(KeyStoreError::AddressNotFound(address.to_owned()))?
            .len())
    }
    async fn get_pq_pre_key_count(&self, address: &DeviceAddress) -> Result<usize, KeyStoreError> {
        Ok(self
            .prekeys
            .get(address)
            .ok_or(KeyStoreError::AddressNotFound(address.to_owned()))?
            .len())
    }
}
