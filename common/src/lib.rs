pub mod error;
mod envelope;

pub use error::LibError;
pub use error::Result;

include!(concat!(env!("OUT_DIR"), "/_includes.rs"));
