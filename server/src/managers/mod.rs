use traits::key_manager::{
    EcPreKeyManager, LastResortPqPreKeyManager, PqPreKeyManager, SignedPreKeyManager,
};

pub mod entities;
pub mod error;
pub mod in_memory;
pub mod postgres;
pub mod traits;

pub trait KeyManagerType: Clone + Send + Sync {
    type EcPreKeyManager: EcPreKeyManager;
    type PqPreKeyManager: PqPreKeyManager;
    type SignedPreKeyManager: SignedPreKeyManager;
    type LastResortPqPreKeyManager: LastResortPqPreKeyManager;
}

#[derive(Clone, Debug, Default)]
pub struct KeyManager<T: KeyManagerType> {
    pub pre_keys: T::EcPreKeyManager,
    pub pq_pre_keys: T::PqPreKeyManager,
    pub signed_pre_keys: T::SignedPreKeyManager,
    pub last_resort_keys: T::LastResortPqPreKeyManager,
}

impl<T: KeyManagerType> KeyManager<T> {
    pub fn new(
        pre_keys: T::EcPreKeyManager,
        pq_pre_keys: T::PqPreKeyManager,
        signed_pre_keys: T::SignedPreKeyManager,
        last_resort_keys: T::LastResortPqPreKeyManager,
    ) -> Self {
        Self {
            pre_keys,
            pq_pre_keys,
            signed_pre_keys,
            last_resort_keys,
        }
    }
}
