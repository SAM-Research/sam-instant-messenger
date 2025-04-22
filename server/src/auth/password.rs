use crate::auth::error::AuthorizationError;
use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use log::debug;
use rand::{CryptoRng, Rng};

#[derive(Clone, bon::Builder, PartialEq, Eq)]
pub struct Password {
    hash: String,
    salt: SaltString,
}

impl Password {
    pub fn generate<T: Rng + CryptoRng + Default>(
        password: String,
        rng: &mut T,
    ) -> Result<Self, AuthorizationError> {
        let argon = Argon2::default();
        let salt = SaltString::generate(rng);
        let hash = argon
            .hash_password(password.as_bytes(), &salt)
            .inspect_err(|e| debug!("{e}"))
            .map_err(|_| AuthorizationError::PasswordHashError)?
            .to_string();
        Ok(Self { hash, salt })
    }

    pub fn verify(&self, password: String) -> Result<(), AuthorizationError> {
        let pwd_hash = PasswordHash::new(&self.hash)
            .inspect_err(|e| debug!("{e}"))
            .map_err(|_| AuthorizationError::PasswordHashError)?;
        let res = Argon2::default().verify_password(password.as_bytes(), &pwd_hash);
        res.inspect_err(|e| debug!("{e}"))
            .map_err(|_| AuthorizationError::WrongPassword)
    }
}
