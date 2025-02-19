pub mod address;
pub mod api;
pub mod error;
mod proto;
pub mod time;

pub use error::LibError;
pub use error::Result;

pub use time::time_now_millis;

include!(concat!(env!("OUT_DIR"), "/_includes.rs"));
