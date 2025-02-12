pub mod account;
pub mod device;
pub mod error;
pub mod inmem;
pub mod postgres;
pub mod traits;

pub trait PublicKey {
    fn serialize(&self) -> Vec<u8>;
    fn deserialize(bytes: &[u8]) -> Self;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdentityKey;

impl PublicKey for IdentityKey {
    fn serialize(&self) -> Vec<u8> {
        todo!()
    }

    fn deserialize(_bytes: &[u8]) -> Self {
        todo!()
    }
}
