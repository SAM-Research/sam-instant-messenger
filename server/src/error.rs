use derive_more::derive::{Display, Error, From};
use sam_common::LibError;

pub type Result<T> = std::result::Result<T, ServerError>;

#[derive(Debug, Display, Error, From)]
pub enum ServerError {
    #[error(ignore)]
    Custom(String),
    Lib(LibError),
}
