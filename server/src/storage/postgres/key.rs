use async_trait::async_trait;
use sam_common::address::DeviceAddress;

use crate::storage::{
    error::KeyStoreError, traits::KeyStore, PreKeyBundle, PreKeyRecord, SignedPreKeyRecord,
};

#[derive(Debug)]
pub struct PostgresKeyStore {}

#[async_trait(?Send)]
impl KeyStore for PostgresKeyStore {
    async fn store_signed_pre_key(
        &mut self,
        _spk: SignedPreKeyRecord,
        _address: &DeviceAddress,
    ) -> Result<(), KeyStoreError> {
        todo!()
    }
    async fn store_last_resort_pq_pre_key(
        &mut self,
        _pq_spk: SignedPreKeyRecord,
        _address: &DeviceAddress,
    ) -> Result<(), KeyStoreError> {
        todo!()
    }
    async fn store_last_resort_ec_pre_key(
        &mut self,
        _pk: PreKeyRecord,
        _owner: &DeviceAddress,
    ) -> Result<(), KeyStoreError> {
        todo!()
    }
    async fn store_one_time_pq_pre_keys(
        &mut self,
        _otpks: Vec<SignedPreKeyRecord>,
        _owner: &DeviceAddress,
    ) -> Result<(), KeyStoreError> {
        todo!()
    }
    async fn store_one_time_ec_pre_keys(
        &mut self,
        _otpks: Vec<PreKeyRecord>,
        _owner: &DeviceAddress,
    ) -> Result<(), KeyStoreError> {
        todo!()
    }

    async fn store_key_bundle(
        &mut self,
        _data: PreKeyBundle,
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
