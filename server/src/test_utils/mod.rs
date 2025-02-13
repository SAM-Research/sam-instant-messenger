#[cfg(test)]
pub(crate) mod cache;
pub(crate) mod user;

use rand::{distributions::Alphanumeric, Rng};

#[cfg(test)]
pub fn random_string(length: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}
