use derive_more::derive::{Display, Error, From};
use sam_common::LibError;

pub type Result<T> = std::result::Result<T, ServerError>;

#[derive(Debug, Display, Error, From)]
pub enum ServerError {
    #[error(ignore)]
    Custom(String),
    Lib(LibError),
    #[display("No connection to Redis cache")]
    NoConnectionToCache,
    /*#[display(fmt = "Could not insert {} into the cache: {}", _0, _1)]
    CacheInsertionError(String, RedisError),
    #[display(fmt = "Could not remove {} from the cache: {}", _0, _1)]
    CacheRemoveError(String, RedisError),
    #[display(fmt = "Failed to communicate with cache: {}", _0)]
    CacheCommunicationError(RedisError),*/
}
