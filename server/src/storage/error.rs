use derive_more::{Display, Error, From};
use std::error::Error as StdError;

type BoxDynError = Box<dyn StdError + 'static + Send + Sync>;

#[derive(Debug, Display, Error)]
pub enum AccountStoreError {
    #[error(ignore)]
    AccountIdTaken(String),
    Database(BoxDynError),
    #[error(ignore)]
    AccountNotFound(String),
}

#[derive(Debug, Display, Error)]
pub enum DeviceStoreError {
    #[error(ignore)]
    AccountNotFound(String),
    #[error(ignore)]
    DeviceIdTaken(u32),
    Database(BoxDynError),
}

#[derive(Debug, Display, From, Error)]
pub enum KeyStoreError {
    Database(BoxDynError),
}

#[derive(Debug, Display, From, Error)]
pub enum MessageStoreError {
    Database(BoxDynError),
}
