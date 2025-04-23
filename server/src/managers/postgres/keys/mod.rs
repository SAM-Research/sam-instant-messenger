use crate::managers::KeyManagerType;

mod ec;
mod last_resort;
mod pq;
mod signed;

pub use ec::PostgresEcPreKeyManager;
pub use last_resort::PostgresLastResortPqPreKeyManager;
pub use pq::PostgresPqPreKeyManager;
pub use signed::PostgresSignedPreKeyManager;

#[derive(Clone)]
pub struct PostgresKeyManager;

impl KeyManagerType for PostgresKeyManager {
    type EcPreKeyManager = PostgresEcPreKeyManager;

    type PqPreKeyManager = PostgresPqPreKeyManager;

    type SignedPreKeyManager = PostgresSignedPreKeyManager;

    type LastResortPqPreKeyManager = PostgresLastResortPqPreKeyManager;
}
