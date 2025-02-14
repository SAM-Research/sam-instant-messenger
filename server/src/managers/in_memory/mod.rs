use uuid::Uuid;

pub mod account;
pub mod device;
pub mod keys;
pub mod message;

pub fn device_key(accunt_id: &Uuid, id: u32) -> String {
    format!("{}.{}", accunt_id, id)
}
