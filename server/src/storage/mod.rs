use async_trait::async_trait;
use bon::Builder;
use error::{AccountStoreError, DeviceStoreError, KeyStoreError, MessageStoreError};
use libsignal_protocol::{Aci, IdentityKey, ProtocolAddress};
use sam_common::sam_message::ServerEnvelope;
use std::error::Error;

pub mod error;
pub mod inmem;
pub mod postgres;

#[derive(Debug, Builder, Clone)]
pub struct Account {
    aci: Aci,
    identity_key: IdentityKey,
}

impl Account {
    pub fn aci(&self) -> &Aci {
        &self.aci
    }
    pub fn identity_key(&self) -> &IdentityKey {
        &self.identity_key
    }
}

#[derive(Debug, Clone)]
pub struct Device {
    id: u32,
}

pub trait DatabaseError: Error {}

pub enum DeviceCapability {}

#[async_trait(?Send)]
pub trait AccountStore {
    async fn add_account(&mut self, account: &Account) -> Result<(), AccountStoreError>;
    async fn get_account(&self, service_id: &Aci) -> Result<Account, AccountStoreError>;
    async fn update_account_identifier(
        &mut self,
        service_id: &Aci,
        new_aci: Aci,
    ) -> Result<(), AccountStoreError>;
    async fn delete_account(&mut self, service_id: &Aci) -> Result<(), AccountStoreError>;
    async fn add_used_device_link_token(
        &mut self,
        device_link_token: String,
    ) -> Result<(), AccountStoreError>;
}

#[async_trait(?Send)]
pub trait DeviceStore {
    async fn add_device(
        &mut self,
        service_id: &Aci,
        device: &Device,
    ) -> Result<(), DeviceStoreError>;
    async fn get_all_devices(&self, service_id: &Aci) -> Result<Vec<Device>, DeviceStoreError>;
    async fn get_device(&self, address: &ProtocolAddress) -> Result<Device, DeviceStoreError>;
    async fn delete_device(&mut self, address: &ProtocolAddress) -> Result<(), DeviceStoreError>;
}

#[async_trait(?Send)]
pub trait MessageStore {
    async fn push_message_queue(
        &mut self,
        address: &ProtocolAddress,
        messages: Vec<ServerEnvelope>,
    ) -> Result<(), MessageStoreError>;

    async fn pop_msg_queue(
        &mut self,
        address: &ProtocolAddress,
    ) -> Result<Vec<ServerEnvelope>, MessageStoreError>;

    async fn count_messages(&self, address: &ProtocolAddress) -> Result<u32, MessageStoreError>;

    async fn get_messages(
        &self,
        address: &ProtocolAddress,
    ) -> Result<Vec<ServerEnvelope>, MessageStoreError>;

    async fn delete_messages(
        &mut self,
        address: &ProtocolAddress,
    ) -> Result<Vec<ServerEnvelope>, MessageStoreError>;
}

#[derive(Debug, Clone)]
pub struct PreKeyRecord;

#[derive(Debug, Clone)]
pub struct SignedPreKeyRecord;

#[derive(Debug, Clone)]
pub struct PreKeyBundle;

#[async_trait(?Send)]
pub trait KeyStore {
    async fn store_signed_pre_key(
        &mut self,
        spk: &SignedPreKeyRecord,
        address: &ProtocolAddress,
    ) -> Result<(), KeyStoreError>;

    async fn store_last_resort_pq_pre_key(
        &mut self,
        pq_spk: &SignedPreKeyRecord,
        address: &ProtocolAddress,
    ) -> Result<(), KeyStoreError>;

    async fn store_last_resort_ec_pre_key(
        &mut self,
        pk: PreKeyRecord,
        owner: &ProtocolAddress,
    ) -> Result<(), KeyStoreError>;

    async fn store_one_time_pq_pre_keys(
        &mut self,
        otpks: Vec<SignedPreKeyRecord>,
        owner: &ProtocolAddress,
    ) -> Result<(), KeyStoreError>;

    async fn store_one_time_ec_pre_keys(
        &mut self,
        otpks: Vec<PreKeyRecord>,
        owner: &ProtocolAddress,
    ) -> Result<(), KeyStoreError>;

    async fn store_key_bundle(
        &mut self,
        data: &PreKeyBundle,
        address: &ProtocolAddress,
    ) -> Result<(), KeyStoreError>;

    async fn get_key_bundle(
        &self,
        address: &ProtocolAddress,
    ) -> Result<PreKeyBundle, KeyStoreError>;

    async fn get_one_time_ec_pre_key_count(
        &self,
        address: &ProtocolAddress,
    ) -> Result<usize, KeyStoreError>;

    async fn get_one_time_pq_pre_key_count(
        &self,
        address: &ProtocolAddress,
    ) -> Result<usize, KeyStoreError>;
}
