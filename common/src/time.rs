use std::time::{SystemTime, UNIX_EPOCH};

pub fn time_now_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Now is later than epoch")
        .as_millis()
}
