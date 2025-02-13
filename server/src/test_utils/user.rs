use libsignal_protocol::{Aci, DeviceId, ProtocolAddress};
use rand::prelude::StdRng;
use rand::{Rng, SeedableRng};
use uuid::Uuid;

pub fn new_protocol_address() -> ProtocolAddress {
    let name = new_aci();
    ProtocolAddress::new(name.service_id_string(), new_device_id())
}

pub fn new_aci() -> Aci {
    Aci::from(new_uuid())
}

pub fn new_device_id() -> DeviceId {
    new_rand_number().into()
}

pub fn new_rand_number() -> u32 {
    StdRng::from_entropy().gen::<u32>()
}

pub fn new_uuid() -> Uuid {
    Uuid::new_v4()
}
