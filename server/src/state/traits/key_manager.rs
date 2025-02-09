use libsignal_protocol::ProtocolAddress;
use sam_common::api::keys::{UploadPreKey, UploadSignedPreKey};

use crate::ServerError;

pub enum KeyType {
    Normal(UploadPreKey),
    NormalSigned(UploadSignedPreKey),
    Quantum(UploadPreKey),
    QuantumSigned(UploadSignedPreKey),
}

impl KeyType {
    pub fn unwrap_signed(self) -> UploadSignedPreKey {
        match self {
            KeyType::Normal(_) => panic!("Called unwrap_signed on a normal key"),
            KeyType::NormalSigned(x) => x,
            KeyType::Quantum(_) => panic!("Called unwrap_signed on a normal key"),
            KeyType::QuantumSigned(x) => x,
        }
    }
    pub fn unwrap(self) -> UploadPreKey {
        match self {
            KeyType::Normal(x) => x,
            KeyType::NormalSigned(_) => panic!("Called unwrap on a singed key"),
            KeyType::Quantum(x) => x,
            KeyType::QuantumSigned(_) => panic!("Called unwrap on a singed key"),
        }
    }
}

pub enum KeyKind {
    Normal,
    NormalSigned,
    Quantum,
    QuantumSigned,
}

#[async_trait::async_trait]
pub trait KeyManager: Send {
    async fn add_key(&mut self, addr: &ProtocolAddress, key: KeyType) -> Result<(), ServerError>;
    async fn get_key(
        &mut self,
        addr: &ProtocolAddress,
        key: KeyKind,
    ) -> Result<KeyType, ServerError>;
    async fn remove_key(
        &mut self,
        addr: &ProtocolAddress,
        key: &KeyType,
    ) -> Result<(), ServerError>;
}
