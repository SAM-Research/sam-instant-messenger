use derive_more::derive::{Display, Error, From};

pub type Result<T> = std::result::Result<T, LibError>;

#[derive(Debug, Display, Error, From)]
pub enum LibError {
    #[error(ignore)]
    Custom(String),
}
