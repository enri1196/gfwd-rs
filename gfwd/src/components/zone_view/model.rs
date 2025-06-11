use crate::fwd_broker::FwdBroker;

#[derive(Debug)]
pub struct ZoneViewModel {
    pub zone_name: String,
    // Add more fields as needed for zone-specific data
}

impl ZoneViewModel {
    pub fn new(zone_name: String) -> Self {
        Self {
            zone_name,
        }
    }
}
