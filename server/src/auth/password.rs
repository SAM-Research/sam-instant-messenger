use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

use crate::ServerError;

#[derive(Clone, bon::Builder, PartialEq, Eq)]
pub struct Password {
    hash: String,
    salt: SaltString,
}

impl Password {
    pub fn generate(password: String) -> Result<Self, ServerError> {
        let argon = Argon2::default();
        let salt = SaltString::generate(&mut OsRng);
        let hash = argon
            .hash_password(password.as_bytes(), &salt)
            .map_err(|_| ServerError::PasswordHashError)?
            .to_string();
        Ok(Self { hash, salt })
    }

    pub fn verify(&self, password: String) -> Result<(), ServerError> {
        let pwd_hash = PasswordHash::new(&self.hash).map_err(|_| ServerError::PasswordHashError)?;
        let res = Argon2::default().verify_password(password.as_bytes(), &pwd_hash);
        res.map_err(|_| ServerError::WrongPassword)
    }
}
