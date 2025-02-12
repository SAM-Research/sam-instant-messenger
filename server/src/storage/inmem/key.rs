use std::collections::HashMap;

use async_trait::async_trait;
use sam_common::{
    address::DeviceAddress,
    api::{EcPreKey, PreKeyBundle, PublishPreKeys, SignedEcPreKey},
};

use crate::storage::{error::KeyStoreError, traits::KeyStore};

#[derive(Debug)]
pub struct InMemoryKeyStore {
    last_resort_prekeys: HashMap<DeviceAddress, EcPreKey>,
    prekeys: HashMap<DeviceAddress, Vec<EcPreKey>>,
    last_resort_pq_prekeys: HashMap<DeviceAddress, SignedEcPreKey>,
    pq_prekeys: HashMap<DeviceAddress, Vec<SignedEcPreKey>>,
    signed_prekeys: HashMap<DeviceAddress, SignedEcPreKey>,
}

#[async_trait(?Send)]
impl KeyStore for InMemoryKeyStore {
    async fn store_signed_pre_key(
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
        pq_spk: SignedEcPreKey,
        address: &DeviceAddress,
    ) -> Result<(), KeyStoreError> {
        self.last_resort_pq_prekeys
            .insert(address.to_owned(), pq_spk.to_owned());
        Ok(())
    }
    async fn store_last_resort_ec_pre_key(
        &mut self,
        pk: EcPreKey,
        owner: &DeviceAddress,
    ) -> Result<(), KeyStoreError> {
        self.last_resort_prekeys
            .insert(owner.to_owned(), pk.to_owned());
        Ok(())
    }
    async fn store_one_time_pq_pre_keys(
        &mut self,
        otpks: Vec<SignedEcPreKey>,
        owner: &DeviceAddress,
    ) -> Result<(), KeyStoreError> {
        self.pq_prekeys.insert(owner.to_owned(), otpks);
        Ok(())
    }
    async fn store_one_time_ec_pre_keys(
        &mut self,
        otpks: Vec<EcPreKey>,
        owner: &DeviceAddress,
    ) -> Result<(), KeyStoreError> {
        self.prekeys.insert(owner.to_owned(), otpks);
        Ok(())
    }

    async fn store_key_bundle(
        &mut self,
        _data: PublishPreKeys,
        _address: &DeviceAddress,
    ) -> Result<(), KeyStoreError> {
        todo!()
    }

    async fn get_key_bundle(
        &self,
        _address: &DeviceAddress,
    ) -> Result<PreKeyBundle, KeyStoreError> {
        todo!()
    }
    async fn get_one_time_ec_pre_key_count(
        &self,
        address: &DeviceAddress,
    ) -> Result<usize, KeyStoreError> {
        Ok(self
            .prekeys
            .get(address)
            .ok_or(KeyStoreError::AddressNotFound(address.to_owned()))?
            .len())
    }
    async fn get_one_time_pq_pre_key_count(
        &self,
        address: &DeviceAddress,
    ) -> Result<usize, KeyStoreError> {
        Ok(self
            .prekeys
            .get(address)
            .ok_or(KeyStoreError::AddressNotFound(address.to_owned()))?
            .len())
    }
}
