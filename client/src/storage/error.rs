use derive_more::derive::{Display, Error};

#[derive(Debug, Display, Error)]
pub enum StoreError {
    #[display("Failed to parse an invalid AccountId: {_0}")]
    #[error(ignore)]
    InvalidAccountId(String),
    NoDeviceId,
    NoAccountId,
    NoPassword,
    NoUsername,
    SendError,
    Database(#[error(not(source))] String),
}
