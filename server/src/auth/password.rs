use crate::auth::error::AuthorizationError;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

#[derive(Clone, bon::Builder, PartialEq, Eq)]
pub struct Password {
    hash: String,
    salt: SaltString,
}

impl Password {
    pub fn generate(password: String) -> Result<Self, AuthorizationError> {
        let argon = Argon2::default();
        let salt = SaltString::generate(&mut OsRng);
        let hash = argon
            .hash_password(password.as_bytes(), &salt)
            .map_err(|_| AuthorizationError::PasswordHashError)?
            .to_string();
        Ok(Self { hash, salt })
    }

    pub fn verify(&self, password: String) -> Result<(), AuthorizationError> {
        let pwd_hash =
            PasswordHash::new(&self.hash).map_err(|_| AuthorizationError::PasswordHashError)?;
        let res = Argon2::default().verify_password(password.as_bytes(), &pwd_hash);
        res.map_err(|_| AuthorizationError::WrongPassword)
    }
}
