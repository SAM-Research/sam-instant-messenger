pub mod error;
mod proto;

pub use error::LibError;
pub use error::Result;

include!(concat!(env!("OUT_DIR"), "/_includes.rs"));
