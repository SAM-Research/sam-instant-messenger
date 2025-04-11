use crate::auth::error::AuthorizationError;
use argon2::{
    password_hash::{
        rand_core::OsRng, Error as Argon2HashError, PasswordHash, PasswordHasher, PasswordVerifier,
        SaltString,
    },
    Argon2,
};
use derive_more::{Deref, From, Into};
use log::debug;

#[derive(Debug, Clone, PartialEq, Eq, From, Into, Deref)]
pub struct Salt(String);

impl<'a> TryFrom<Salt> for SaltString {
    type Error = Argon2HashError;

    fn try_from(value: Salt) -> Result<Self, Self::Error> {
        SaltString::from_b64(&*value)
    }
}

#[derive(Clone, bon::Builder, PartialEq, Eq)]
pub struct Password {
    hash: String,
    #[builder(into)]
    salt: Salt,
}

impl Password {
    pub fn hash(&self) -> &String {
        &self.hash
    }

    pub fn salt(&self) -> &Salt {
        &self.salt
    }

    pub fn generate(password: String) -> Result<Self, AuthorizationError> {
        let argon = Argon2::default();
        let salt = SaltString::generate(&mut OsRng);
        let hash = argon
            .hash_password(password.as_bytes(), &salt)
            .inspect_err(|e| debug!("{e}"))
            .map_err(|_| AuthorizationError::PasswordHashError)?
            .to_string();
        let salt = salt.to_string().into();
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
