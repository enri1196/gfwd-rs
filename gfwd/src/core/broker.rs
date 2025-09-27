use std::collections::HashMap;

use gfwd_bus::config_firewalld1::ZoneSettings as ZoneSettingsBus;
use relm4::tokio::sync::OnceCell;
use zbus::Connection;

use crate::core::error::GfwdError;
use crate::models::zone::{ZoneSettings, ZoneTarget};

pub struct FwdBroker {
    conn: Connection,
}

impl PartialEq for FwdBroker {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

// Static object holding the sender side of the channel
static BROKER: OnceCell<FwdBroker> = OnceCell::const_new();

impl FwdBroker {
    pub async fn get_broker() -> &'static FwdBroker {
        BROKER
            .get_or_init(|| async move {
                let conn = Connection::system().await.unwrap();
                FwdBroker { conn }
            })
            .await
    }

    #[allow(unused)]
    pub fn conn(&self) -> &Connection {
        &self.conn
    }

    /// Get all zones
    pub async fn get_zones(&self) -> Result<Vec<String>, GfwdError> {
        let cfg = gfwd_bus::config_firewalld1::ConfigFirewalld1Proxy::new(&self.conn).await?;
        Ok(cfg.get_zone_names().await?)
    }

    pub async fn get_active_zones(
        &self,
    ) -> Result<HashMap<String, HashMap<String, Vec<String>>>, GfwdError> {
        let zone = gfwd_bus::zone::ZoneProxy::new(&self.conn).await?;
        Ok(zone.get_active_zones().await?)
    }

    /// Get the default zone
    pub async fn get_default_zone(&self) -> Result<String, GfwdError> {
        let cfg = gfwd_bus::config_firewalld1::ConfigFirewalld1Proxy::new(&self.conn).await?;
        Ok(cfg.default_zone().await?)
    }

    pub async fn get_zone_settings(&self, zone_name: &str) -> Result<ZoneSettings, GfwdError> {
        let cfg = gfwd_bus::config_firewalld1::ConfigFirewalld1Proxy::new(&self.conn).await?;
        let path = cfg.get_zone_by_name(zone_name).await?;
        let proxy = gfwd_bus::config_zone::ConfigZoneProxy::builder(&self.conn)
            .path(path.as_str())?
            .build()
            .await?;

        // Fetch each setting individually for maximum compatibility.
        // This avoids calling getSettings or getSettings2, which may not exist.
        let name = zone_name.to_string();
        let version = proxy.get_version().await?;
        let description = proxy.get_description().await?;
        let target_str = proxy.get_target().await?;
        let services = proxy.get_services().await?;
        let ports = proxy.get_ports().await?;
        let icmp_blocks = proxy.get_icmp_blocks().await?;
        let masquerade = proxy.get_masquerade().await?;
        let forward_ports = proxy.get_forward_ports().await?;
        let interfaces = proxy.get_interfaces().await?;
        let sources = proxy.get_sources().await?;
        let rich_rules = proxy.get_rich_rules().await?;
        let protocols = proxy.get_protocols().await?;
        let source_ports = proxy.get_source_ports().await?;

        let settings = ZoneSettings {
            version,
            name,
            description,
            unused: false,
            target: match target_str.as_str() {
                "ACCEPT" => ZoneTarget::Accept,
                "DROP" => ZoneTarget::Drop,
                "REJECT" => ZoneTarget::Reject,
                _ => ZoneTarget::Default,
            },
            services,
            ports,
            icmp_blocks,
            masquerade,
            forward_ports,
            interfaces,
            sources,
            rich_rules,
            protocols,
            source_ports,
        };

        Ok(settings)
    }

    /// Add a new zone with the given settings
    pub async fn add_zone(&self, settings: ZoneSettings) -> Result<(), GfwdError> {
        let name = settings.name.clone();
        let zone_settings: ZoneSettingsBus = (
            settings.version,            // version
            settings.name,               // name
            settings.description,        // description
            settings.unused,             // UNUSED
            settings.target.to_string(), // target
            settings.services,           // services
            settings.ports,              // ports
            settings.icmp_blocks,        // icmp-blocks
            settings.masquerade,         // masquerade
            settings.forward_ports,      // forward-ports
            settings.interfaces,         // interfaces
            settings.sources,
            settings.rich_rules,
            settings.protocols,
            settings.source_ports,
        );
        let cfg = gfwd_bus::config_firewalld1::ConfigFirewalld1Proxy::new(&self.conn).await?;
        Ok(cfg
            .add_zone(name.as_str(), &zone_settings)
            .await
            .map(|_| ())?)
    }

    pub async fn remove_zone(&self, zone_name: &str) -> Result<(), GfwdError> {
        let cfg = gfwd_bus::config_firewalld1::ConfigFirewalld1Proxy::new(&self.conn).await?;
        let selected_zone = cfg.get_zone_by_name(zone_name).await?;

        let proxy_zone =
            gfwd_bus::config_zone::ConfigZoneProxy::new(&self.conn, selected_zone).await?;
        Ok(proxy_zone.remove().await?)
    }

    pub async fn add_port(
        &self,
        zone_name: &str,
        port: &str,
        protocol: &str,
    ) -> Result<(), GfwdError> {
        let cfg = gfwd_bus::config_firewalld1::ConfigFirewalld1Proxy::new(&self.conn).await?;
        let path = cfg.get_zone_by_name(&zone_name).await?;
        let zone = gfwd_bus::config_zone::ConfigZoneProxy::builder(&self.conn)
            .path(path.as_str())?
            .build()
            .await?;
        Ok(zone.add_port(&port, &protocol).await?)
    }

    pub async fn remove_port(
        &self,
        zone_name: &str,
        port: &str,
        protocol: &str,
    ) -> Result<(), GfwdError> {
        let cfg = gfwd_bus::config_firewalld1::ConfigFirewalld1Proxy::new(&self.conn).await?;
        let path = cfg.get_zone_by_name(&zone_name).await?;
        let zone = gfwd_bus::config_zone::ConfigZoneProxy::builder(&self.conn)
            .path(path.as_str())?
            .build()
            .await?;
        Ok(zone.remove_port(&port, &protocol).await?)
    }

    pub async fn add_forward_port(
        &self,
        zone_name: &str,
        port: &str,
        protocol: &str,
        to_port: &str,
        to_addr: &str,
    ) -> Result<(), GfwdError> {
        let cfg = gfwd_bus::config_firewalld1::ConfigFirewalld1Proxy::new(&self.conn).await?;
        let path = cfg.get_zone_by_name(&zone_name).await?;
        let zone = gfwd_bus::config_zone::ConfigZoneProxy::builder(&self.conn)
            .path(path.as_str())?
            .build()
            .await?;
        Ok(zone
            .add_forward_port(&port, &protocol, to_port, to_addr)
            .await?)
    }

    pub async fn remove_forward_port(
        &self,
        zone_name: &str,
        port: &str,
        protocol: &str,
        to_port: &str,
        to_addr: &str,
    ) -> Result<(), GfwdError> {
        let cfg = gfwd_bus::config_firewalld1::ConfigFirewalld1Proxy::new(&self.conn).await?;
        let path = cfg.get_zone_by_name(&zone_name).await?;
        let zone = gfwd_bus::config_zone::ConfigZoneProxy::builder(&self.conn)
            .path(path.as_str())?
            .build()
            .await?;
        Ok(zone
            .remove_forward_port(&port, &protocol, to_port, to_addr)
            .await?)
    }

    pub async fn is_firewalld_active(&self) -> Result<bool, GfwdError> {
        let mgr = gfwd_bus::systemd::ManagerProxy::new(&self.conn).await?;
        let unit_path = mgr.get_unit("firewalld.service").await?;
        let unit = gfwd_bus::systemd::UnitProxy::builder(&self.conn)
            .path(unit_path.as_str())?
            .build()
            .await?;
        let active = unit.active_state().await?;
        Ok(active == "active" || active == "activating")
    }

    pub async fn start_firewalld(&self) -> Result<(), GfwdError> {
        let mgr = gfwd_bus::systemd::ManagerProxy::new(&self.conn).await?;
        let _job = mgr.start_unit("firewalld.service", "replace").await?;
        Ok(())
    }

    pub async fn stop_firewalld(&self) -> Result<(), GfwdError> {
        let mgr = gfwd_bus::systemd::ManagerProxy::new(&self.conn).await?;
        let _job = mgr.stop_unit("firewalld.service", "replace").await?;
        Ok(())
    }
}
