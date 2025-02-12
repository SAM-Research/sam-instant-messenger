use async_trait::async_trait;
use sam_common::{
    address::DeviceAddress,
    api::{keys::PublishPreKeys, EcPreKey, PreKeyBundle, SignedEcPreKey},
};

use crate::storage::{error::KeyStoreError, traits::KeyStore};

#[derive(Debug)]
pub struct PostgresKeyStore {}

#[async_trait(?Send)]
impl KeyStore for PostgresKeyStore {
    async fn store_signed_pre_key(
        &mut self,
        _spk: SignedEcPreKey,
        _address: &DeviceAddress,
    ) -> Result<(), KeyStoreError> {
        todo!()
    }
    async fn store_last_resort_pq_pre_key(
        &mut self,
        _pq_spk: SignedEcPreKey,
        _address: &DeviceAddress,
    ) -> Result<(), KeyStoreError> {
        todo!()
    }
    async fn store_last_resort_ec_pre_key(
        &mut self,
        _pk: EcPreKey,
        _owner: &DeviceAddress,
    ) -> Result<(), KeyStoreError> {
        todo!()
    }
    async fn store_one_time_pq_pre_keys(
        &mut self,
        _otpks: Vec<SignedEcPreKey>,
        _owner: &DeviceAddress,
    ) -> Result<(), KeyStoreError> {
        todo!()
    }
    async fn store_one_time_ec_pre_keys(
        &mut self,
        _otpks: Vec<EcPreKey>,
        _owner: &DeviceAddress,
    ) -> Result<(), KeyStoreError> {
        todo!()
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
        _address: &DeviceAddress,
    ) -> Result<usize, KeyStoreError> {
        todo!()
    }
    async fn get_one_time_pq_pre_key_count(
        &self,
        _address: &DeviceAddress,
    ) -> Result<usize, KeyStoreError> {
        todo!()
    }
}
