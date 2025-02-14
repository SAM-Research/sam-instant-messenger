use derive_more::derive::{From, Into};
use rand::Rng;
use serde::{Deserialize, Serialize};

const REGISTRATION_ID_MAX: u32 = 16383;

#[derive(
    Copy,
    Clone,
    Debug,
    Hash,
    Eq,
    PartialEq,
    PartialOrd,
    Ord,
    From,
    Into,
    Default,
    Serialize,
    Deserialize,
)]
pub struct RegistrationId(u32);

impl RegistrationId {
    pub fn generate<R: Rng>(csprng: &mut R) -> Self {
        RegistrationId(csprng.gen_range(1..REGISTRATION_ID_MAX))
    }
}
