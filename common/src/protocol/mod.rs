use crate::{address::DeviceId, sam_message::DeviceList};

pub mod error;

impl From<Vec<DeviceId>> for DeviceList {
    fn from(value: Vec<DeviceId>) -> Self {
        Self {
            ids: value.into_iter().map(|id| id.into()).collect(),
        }
    }
}
