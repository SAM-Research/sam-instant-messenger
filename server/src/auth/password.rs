use crate::auth::error::AuthorizationError;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use log::debug;

#[derive(Clone, bon::Builder, PartialEq, Eq)]
pub struct Password {
    hash: String,
}

impl Password {
    pub fn hash(&self) -> &String {
        &self.hash
    }
    pub fn generate(password: String) -> Result<Self, AuthorizationError> {
        let argon = Argon2::default();
        let salt = SaltString::generate(&mut OsRng);
        let hash = argon
            .hash_password(password.as_bytes(), &salt)
            .inspect_err(|e| debug!("{e}"))
            .map_err(|_| AuthorizationError::PasswordHashError)?
            .to_string();
        Ok(Self { hash })
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
