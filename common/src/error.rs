use derive_more::derive::{Display, Error};

pub type Result<T> = std::result::Result<T, LibError>;

#[derive(Debug, Display, Error)]
pub enum LibError {
    #[error(ignore)]
    Custom(String),
    #[error(ignore)]
    AuthorizationError(String),
}
