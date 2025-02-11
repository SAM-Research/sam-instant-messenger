use async_trait::async_trait;
use sam_common::{
    address::{AccountId, DeviceAddress},
    sam_message::ServerEnvelope,
};

use super::{
    account::Account,
    device::Device,
    error::{AccountStoreError, DeviceStoreError, KeyStoreError, MessageStoreError},
    PreKeyBundle, PreKeyRecord, SignedPreKeyRecord,
};

#[async_trait(?Send)]
pub trait AccountStore {
    async fn add_account(&mut self, account: Account) -> Result<(), AccountStoreError>;
    async fn get_account(&self, account_id: &AccountId) -> Result<Account, AccountStoreError>;
    async fn update_account_identifier(
        &mut self,
        current_id: &AccountId,
        new_id: &AccountId,
    ) -> Result<(), AccountStoreError>;
    async fn delete_account(&mut self, account_id: &AccountId) -> Result<(), AccountStoreError>;
    async fn add_used_device_link_token(
        &mut self,
        device_link_token: String,
    ) -> Result<(), AccountStoreError>;
}

#[async_trait(?Send)]
pub trait DeviceStore {
    async fn add_device(
        &mut self,
        device: Device,
        account_id: &AccountId,
    ) -> Result<(), DeviceStoreError>;
    async fn get_all_devices(
        &self,
        account_id: &AccountId,
    ) -> Result<Vec<Device>, DeviceStoreError>;
    async fn get_device(&self, address: &DeviceAddress) -> Result<Device, DeviceStoreError>;
    async fn delete_device(&mut self, address: &DeviceAddress) -> Result<(), DeviceStoreError>;
}

#[async_trait(?Send)]
pub trait MessageStore {
    async fn push_message_queue(
        &mut self,
        messages: Vec<ServerEnvelope>,
        address: &DeviceAddress,
    ) -> Result<(), MessageStoreError>;

    async fn pop_msg_queue(
        &mut self,
        address: &DeviceAddress,
    ) -> Result<Vec<ServerEnvelope>, MessageStoreError>;

    async fn count_messages(&self, address: &DeviceAddress) -> Result<u32, MessageStoreError>;

    async fn get_messages(
        &self,
        address: &DeviceAddress,
    ) -> Result<Vec<ServerEnvelope>, MessageStoreError>;

    async fn delete_messages(
        &mut self,
        address: &DeviceAddress,
    ) -> Result<Vec<ServerEnvelope>, MessageStoreError>;
}

#[async_trait(?Send)]
pub trait KeyStore {
    async fn store_signed_pre_key(
        &mut self,
        spk: SignedPreKeyRecord,
        address: &DeviceAddress,
    ) -> Result<(), KeyStoreError>;

    async fn store_last_resort_pq_pre_key(
        &mut self,
        pq_spk: SignedPreKeyRecord,
        address: &DeviceAddress,
    ) -> Result<(), KeyStoreError>;

    async fn store_last_resort_ec_pre_key(
        &mut self,
        pk: PreKeyRecord,
        address: &DeviceAddress,
    ) -> Result<(), KeyStoreError>;

    async fn store_one_time_pq_pre_keys(
        &mut self,
        otpks: Vec<SignedPreKeyRecord>,
        address: &DeviceAddress,
    ) -> Result<(), KeyStoreError>;

    async fn store_one_time_ec_pre_keys(
        &mut self,
        otpks: Vec<PreKeyRecord>,
        address: &DeviceAddress,
    ) -> Result<(), KeyStoreError>;

    async fn store_key_bundle(
        &mut self,
        data: PreKeyBundle,
        address: &DeviceAddress,
    ) -> Result<(), KeyStoreError>;

    async fn get_key_bundle(&self, address: &DeviceAddress) -> Result<PreKeyBundle, KeyStoreError>;

    async fn get_one_time_ec_pre_key_count(
        &self,
        address: &DeviceAddress,
    ) -> Result<usize, KeyStoreError>;

    async fn get_one_time_pq_pre_key_count(
        &self,
        address: &DeviceAddress,
    ) -> Result<usize, KeyStoreError>;
}
