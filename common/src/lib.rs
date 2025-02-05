mod envelope;
pub mod error;

pub use error::LibError;
pub use error::Result;

use libsignal_protocol::Timestamp;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn time_now() -> Timestamp {
    Timestamp::from_epoch_millis(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Now is later than epoch")
            .as_millis()
            .try_into()
            .expect("Living in the future is not allowed"),
    )
}

include!(concat!(env!("OUT_DIR"), "/_includes.rs"));
