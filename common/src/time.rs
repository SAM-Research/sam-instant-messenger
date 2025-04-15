use std::time::{SystemTime, UNIX_EPOCH};

use libsignal_protocol::Timestamp;

pub fn time_now_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Now is later than epoch")
        .as_millis()
}

pub fn signal_time_now() -> Timestamp {
    Timestamp::from_epoch_millis(
        time_now_millis()
            .try_into()
            .expect("Living in the future is not allowed"),
    )
}
