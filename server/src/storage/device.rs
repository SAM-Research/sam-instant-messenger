use sam_common::address::DeviceId;

#[derive(Debug, Clone)]
pub struct Device {
    id: DeviceId,
}

impl Device {
    pub fn id(&self) -> DeviceId {
        self.id
    }
}
