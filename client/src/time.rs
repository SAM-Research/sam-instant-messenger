use libsignal_protocol::Timestamp;
use sam_common::time_now_millis;
pub fn time_now() -> Timestamp {
    Timestamp::from_epoch_millis(
        time_now_millis()
            .try_into()
            .expect("Living in the future is not allowed"),
    )
}
