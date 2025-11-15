use std::collections::HashMap;

use async_trait::async_trait;
use relm4::tokio::sync::RwLock;

use crate::error::GfwdError;

use super::backend::{FirewalldBackend, ZoneSettings, ZoneTarget};

pub(super) struct MockFirewalld {
    state: RwLock<MockState>,
}

struct MockState {
    zones: HashMap<String, ZoneSettings>,
    active_zones: HashMap<String, HashMap<String, Vec<String>>>,
    default_zone: String,
    firewalld_active: bool,
}

impl MockFirewalld {
    pub fn new() -> Self {
        let mut zones = HashMap::new();
        let default = ZoneSettings {
            version: "1".to_string(),
            name: "public".to_string(),
            description: "Default public zone".to_string(),
            unused: false,
            target: ZoneTarget::Default,
            services: vec!["ssh".to_string()],
            ports: vec![("22".to_string(), "tcp".to_string())],
            icmp_blocks: vec![],
            masquerade: false,
            forward_ports: vec![],
            interfaces: vec!["en0".to_string()],
            sources: vec!["0.0.0.0/0".to_string()],
            rich_rules: vec![],
            protocols: vec![],
            source_ports: vec![],
        };
        zones.insert(default.name.clone(), default.clone());
        let default_name = default.name.clone();

        let mut active_zones = HashMap::new();
        active_zones.insert(
            default_name.clone(),
            HashMap::from([
                ("interfaces".to_string(), default.interfaces.clone()),
                ("sources".to_string(), default.sources.clone()),
            ]),
        );

        Self {
            state: RwLock::new(MockState {
                zones,
                active_zones,
                default_zone: default_name,
                firewalld_active: true,
            }),
        }
    }
}

#[async_trait]
impl FirewalldBackend for MockFirewalld {
    async fn get_zones(&self) -> Result<Vec<String>, GfwdError> {
        let state = self.state.read().await;
        let mut zones: Vec<_> = state.zones.keys().cloned().collect();
        zones.sort();
        Ok(zones)
    }

    async fn get_active_zones(
        &self,
    ) -> Result<HashMap<String, HashMap<String, Vec<String>>>, GfwdError> {
        let state = self.state.read().await;
        Ok(state.active_zones.clone())
    }

    async fn get_default_zone(&self) -> Result<String, GfwdError> {
        let state = self.state.read().await;
        Ok(state.default_zone.clone())
    }

    async fn get_zone_settings(&self, zone_name: &str) -> Result<ZoneSettings, GfwdError> {
        let state = self.state.read().await;
        state
            .zones
            .get(zone_name)
            .cloned()
            .ok_or_else(|| GfwdError::Validation(format!("Zone '{zone_name}' not found")))
    }

    async fn add_zone(&self, settings: ZoneSettings) -> Result<(), GfwdError> {
        let mut state = self.state.write().await;
        if state.zones.contains_key(&settings.name) {
            return Err(GfwdError::Validation(format!(
                "Zone '{}' already exists",
                settings.name
            )));
        }
        if state.zones.is_empty() {
            state.default_zone = settings.name.clone();
        }
        state.active_zones.insert(settings.name.clone(), HashMap::new());
        state.zones.insert(settings.name.clone(), settings);
        Ok(())
    }

    async fn remove_zone(&self, zone_name: &str) -> Result<(), GfwdError> {
        let mut state = self.state.write().await;
        if state.zones.remove(zone_name).is_none() {
            return Err(GfwdError::Validation(format!(
                "Zone '{}' does not exist",
                zone_name
            )));
        }
        state.active_zones.remove(zone_name);
        if state.default_zone == zone_name {
            state.default_zone = state
                .zones
                .keys()
                .next()
                .cloned()
                .unwrap_or_else(|| "public".to_string());
        }
        Ok(())
    }

    async fn add_port(
        &self,
        zone_name: &str,
        port: &str,
        protocol: &str,
    ) -> Result<(), GfwdError> {
        let mut state = self.state.write().await;
        let zone = state
            .zones
            .get_mut(zone_name)
            .ok_or_else(|| GfwdError::Validation(format!("Zone '{zone_name}' not found")))?;
        if !zone
            .ports
            .iter()
            .any(|(existing_port, existing_proto)| existing_port == port && existing_proto == protocol)
        {
            zone.ports.push((port.to_string(), protocol.to_string()));
        }
        Ok(())
    }

    async fn remove_port(
        &self,
        zone_name: &str,
        port: &str,
        protocol: &str,
    ) -> Result<(), GfwdError> {
        let mut state = self.state.write().await;
        let zone = state
            .zones
            .get_mut(zone_name)
            .ok_or_else(|| GfwdError::Validation(format!("Zone '{zone_name}' not found")))?;
        zone.ports.retain(|(p, proto)| !(p == port && proto == protocol));
        Ok(())
    }

    async fn add_forward_port(
        &self,
        zone_name: &str,
        port: &str,
        protocol: &str,
        to_port: &str,
        to_addr: &str,
    ) -> Result<(), GfwdError> {
        let mut state = self.state.write().await;
        let zone = state
            .zones
            .get_mut(zone_name)
            .ok_or_else(|| GfwdError::Validation(format!("Zone '{zone_name}' not found")))?;
        if !zone.forward_ports.iter().any(|(p, proto, tp, ta)| {
            p == port && proto == protocol && tp == to_port && ta == to_addr
        }) {
            zone.forward_ports.push((
                port.to_string(),
                protocol.to_string(),
                to_port.to_string(),
                to_addr.to_string(),
            ));
        }
        Ok(())
    }

    async fn remove_forward_port(
        &self,
        zone_name: &str,
        port: &str,
        protocol: &str,
        to_port: &str,
        to_addr: &str,
    ) -> Result<(), GfwdError> {
        let mut state = self.state.write().await;
        let zone = state
            .zones
            .get_mut(zone_name)
            .ok_or_else(|| GfwdError::Validation(format!("Zone '{zone_name}' not found")))?;
        zone.forward_ports.retain(|(p, proto, tp, ta)| {
            !(p == port && proto == protocol && tp == to_port && ta == to_addr)
        });
        Ok(())
    }

    async fn is_firewalld_active(&self) -> Result<bool, GfwdError> {
        let state = self.state.read().await;
        Ok(state.firewalld_active)
    }

    async fn start_firewalld(&self) -> Result<(), GfwdError> {
        let mut state = self.state.write().await;
        state.firewalld_active = true;
        Ok(())
    }

    async fn stop_firewalld(&self) -> Result<(), GfwdError> {
        let mut state = self.state.write().await;
        state.firewalld_active = false;
        Ok(())
    }
}
