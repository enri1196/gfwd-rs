use std::collections::HashMap;

use crate::fwd_broker::FwdBroker;

#[derive(Debug, Clone, PartialEq)]
pub struct Zone {
    pub name: String,
    pub is_default: bool,
    pub is_active: bool,
    pub interfaces: Vec<String>,
}

impl Zone {
    pub fn new(name: String) -> Self {
        Self {
            name,
            is_default: false,
            is_active: false,
            interfaces: Vec::new(),
        }
    }
}

pub struct SidebarModel {
    broker: &'static FwdBroker,
    zones: HashMap<String, Zone>,
    default_zone: Option<String>,
}

impl SidebarModel {
    pub async fn new() -> Self {
        Self {
            broker: FwdBroker::get_broker().await,
            zones: HashMap::new(),
            default_zone: None,
        }
    }

    pub async fn add_zone(&mut self, name: &str) {
        self.zones
            .insert(name.to_string(), Zone::new(name.to_string()));
    }

    pub fn set_default_zone(&mut self, name: &str) -> Option<()> {
        if self.zones.contains_key(name) {
            // Reset previous default zone if any
            if let Some(prev_default) = &self.default_zone {
                if let Some(zone) = self.zones.get_mut(prev_default) {
                    zone.is_default = false;
                }
            }

            // Set new default zone
            if let Some(zone) = self.zones.get_mut(name) {
                zone.is_default = true;
                self.default_zone = Some(name.to_string());
                return Some(());
            }
        }
        None
    }

    pub fn set_zone_active(&mut self, name: &str, active: bool) -> Option<()> {
        if let Some(zone) = self.zones.get_mut(name) {
            zone.is_active = active;
            Some(())
        } else {
            None
        }
    }

    pub fn set_zone_interfaces(&mut self, name: &str, interfaces: Vec<String>) -> Option<()> {
        if let Some(zone) = self.zones.get_mut(name) {
            zone.interfaces = interfaces;
            Some(())
        } else {
            None
        }
    }

    pub async fn get_zones(&self) -> Vec<Zone> {
        let zone_names = self.broker.get_zones().await.unwrap();
        let zones = zone_names
            .iter()
            .map(|zone_name| Zone::new(zone_name.clone()))
            .collect();
        zones
    }

    pub async fn get_zone(&self, name: &str) -> Option<&Zone> {
        // let zone = self.broker.get_zones(name).await.unwrap();
        // // let zone = Zone::from(&zone);
        // self.zones.get(name)
        None
    }
}
