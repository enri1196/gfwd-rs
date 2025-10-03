use std::collections::HashMap;

use gfwd_bus::config_firewalld1::ZoneSettings as ZoneSettingsBus;
use relm4::gtk::glib;
use relm4::tokio::sync::OnceCell;
use zbus::Connection;

use crate::core::error::GfwdError;
use crate::models::icmp::IcmpType;
use crate::models::ipset::IPSetSettings;
use crate::models::zone::{ZoneSettings, ZoneTarget};

#[derive(Debug)]
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

    /// Add masquerading to a zone
    pub async fn add_masquerade(&self, zone_name: &str) -> Result<(), GfwdError> {
        let cfg = gfwd_bus::config_firewalld1::ConfigFirewalld1Proxy::new(&self.conn).await?;
        let path = cfg.get_zone_by_name(zone_name).await?;
        let proxy = gfwd_bus::config_zone::ConfigZoneProxy::builder(&self.conn)
            .path(path.as_str())?
            .build()
            .await?;
        proxy.add_masquerade().await?;
        Ok(())
    }

    /// Remove masquerading from a zone
    pub async fn remove_masquerade(&self, zone_name: &str) -> Result<(), GfwdError> {
        let cfg = gfwd_bus::config_firewalld1::ConfigFirewalld1Proxy::new(&self.conn).await?;
        let path = cfg.get_zone_by_name(zone_name).await?;
        let proxy = gfwd_bus::config_zone::ConfigZoneProxy::builder(&self.conn)
            .path(path.as_str())?
            .build()
            .await?;
        proxy.remove_masquerade().await?;
        Ok(())
    }

    /// Add a service to a zone
    pub async fn add_service(&self, zone_name: &str, service: &str) -> Result<(), GfwdError> {
        let cfg = gfwd_bus::config_firewalld1::ConfigFirewalld1Proxy::new(&self.conn).await?;
        let path = cfg.get_zone_by_name(zone_name).await?;
        let proxy = gfwd_bus::config_zone::ConfigZoneProxy::builder(&self.conn)
            .path(path.as_str())?
            .build()
            .await?;
        proxy.add_service(service).await?;
        Ok(())
    }

    /// Remove a service from a zone
    pub async fn remove_service(&self, zone_name: &str, service: &str) -> Result<(), GfwdError> {
        let cfg = gfwd_bus::config_firewalld1::ConfigFirewalld1Proxy::new(&self.conn).await?;
        let path = cfg.get_zone_by_name(zone_name).await?;
        let proxy = gfwd_bus::config_zone::ConfigZoneProxy::builder(&self.conn)
            .path(path.as_str())?
            .build()
            .await?;
        proxy.remove_service(service).await?;
        Ok(())
    }

    /// Get all available services
    pub async fn get_services(&self) -> Result<Vec<String>, GfwdError> {
        let cfg = gfwd_bus::config_firewalld1::ConfigFirewalld1Proxy::new(&self.conn).await?;
        let service_paths = cfg.list_services().await?;
        // Get actual service names by querying each service object
        let mut services = Vec::new();
        for path in service_paths {
            if let Ok(service_proxy) =
                gfwd_bus::config_service::ConfigServiceProxy::builder(&self.conn)
                    .path(path.as_str())
                    .unwrap()
                    .build()
                    .await
            {
                if let Ok(name) = service_proxy.get_short().await {
                    services.push(name);
                }
            }
        }
        services.sort();
        Ok(services)
    }

    /// Set ICMP block inversion for a zone
    pub async fn set_icmp_block_inversion(
        &self,
        zone_name: &str,
        enabled: bool,
    ) -> Result<(), GfwdError> {
        let cfg = gfwd_bus::config_firewalld1::ConfigFirewalld1Proxy::new(&self.conn).await?;
        let path = cfg.get_zone_by_name(zone_name).await?;
        let proxy = gfwd_bus::config_zone::ConfigZoneProxy::builder(&self.conn)
            .path(path.as_str())?
            .build()
            .await?;
        proxy.set_icmp_block_inversion(enabled).await?;
        Ok(())
    }

    /// Get all available ICMP types
    pub async fn get_icmp_types(&self) -> Result<Vec<IcmpType>, GfwdError> {
        let cfg = gfwd_bus::config_firewalld1::ConfigFirewalld1Proxy::new(&self.conn).await?;
        let icmp_type_names = cfg.get_icmp_type_names().await?;

        let mut icmp_types = Vec::new();
        for name in icmp_type_names {
            // Get the object path for this ICMP type
            if let Ok(path) = cfg.get_icmp_type_by_name(&name).await {
                // Create a proxy for this specific ICMP type
                if let Ok(proxy) =
                    gfwd_bus::config_icmptype::ConfigIcmpTypeProxy::builder(&self.conn)
                        .path(path.as_str())
                        .unwrap()
                        .build()
                        .await
                {
                    // Get the description, fallback to name if description fails
                    let description = proxy
                        .get_description()
                        .await
                        .unwrap_or_else(|_| name.clone());
                    icmp_types.push(IcmpType::new(name, description));
                }
            }
        }

        // Sort by name for consistent ordering
        icmp_types.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(icmp_types)
    }

    /// Add an ICMP block to a zone
    pub async fn add_icmp_block(&self, zone_name: &str, icmp_type: &str) -> Result<(), GfwdError> {
        let cfg = gfwd_bus::config_firewalld1::ConfigFirewalld1Proxy::new(&self.conn).await?;
        let path = cfg.get_zone_by_name(zone_name).await?;
        let proxy = gfwd_bus::config_zone::ConfigZoneProxy::builder(&self.conn)
            .path(path.as_str())?
            .build()
            .await?;
        proxy.add_icmp_block(icmp_type).await?;
        Ok(())
    }

    /// Remove an ICMP block from a zone
    pub async fn remove_icmp_block(
        &self,
        zone_name: &str,
        icmp_type: &str,
    ) -> Result<(), GfwdError> {
        let cfg = gfwd_bus::config_firewalld1::ConfigFirewalld1Proxy::new(&self.conn).await?;
        let path = cfg.get_zone_by_name(zone_name).await?;
        let proxy = gfwd_bus::config_zone::ConfigZoneProxy::builder(&self.conn)
            .path(path.as_str())?
            .build()
            .await?;
        proxy.remove_icmp_block(icmp_type).await?;
        Ok(())
    }

    /// Get all available network interfaces from the system using NetworkManager
    pub async fn get_interfaces(&self) -> Result<Vec<String>, GfwdError> {
        // Try NetworkManager first
        match self.get_interfaces_from_networkmanager().await {
            Ok(interfaces) if !interfaces.is_empty() => Ok(interfaces),
            Ok(_) | Err(_) => {
                // Fallback to reading from /sys/class/net if NetworkManager fails or returns empty
                self.get_interfaces_from_sysfs().await
            }
        }
    }

    /// Get network interfaces from NetworkManager D-Bus API
    async fn get_interfaces_from_networkmanager(&self) -> Result<Vec<String>, GfwdError> {
        let nm_proxy = gfwd_bus::network_manager::NetworkManagerProxy::new(&self.conn).await?;
        let device_paths = nm_proxy.get_devices().await?;

        let mut interfaces = Vec::new();
        for device_path in device_paths {
            let device_proxy = gfwd_bus::network_manager::DeviceProxy::builder(&self.conn)
                .path(device_path.as_str())?
                .build()
                .await?;

            if let Ok(interface_name) = device_proxy.interface().await {
                // Filter out loopback and virtual interfaces
                if interface_name != "lo"
                    && !interface_name.starts_with("docker")
                    && !interface_name.starts_with("veth")
                    && !interface_name.starts_with("br-")
                {
                    interfaces.push(interface_name);
                }
            }
        }

        // Sort interfaces for consistent ordering
        interfaces.sort();
        Ok(interfaces)
    }

    /// Fallback method to get network interfaces from /sys/class/net
    async fn get_interfaces_from_sysfs(&self) -> Result<Vec<String>, GfwdError> {
        use std::fs;

        match fs::read_dir("/sys/class/net") {
            Ok(entries) => {
                let mut interfaces = Vec::new();
                for entry in entries {
                    if let Ok(entry) = entry {
                        if let Some(name) = entry.file_name().to_str() {
                            // Filter out loopback and other virtual interfaces
                            if name != "lo"
                                && !name.starts_with("docker")
                                && !name.starts_with("veth")
                                && !name.starts_with("br-")
                            {
                                interfaces.push(name.to_string());
                            }
                        }
                    }
                }
                // Sort interfaces for consistent ordering
                interfaces.sort();
                Ok(interfaces)
            }
            Err(e) => {
                glib::g_log!(
                    glib::LogLevel::Warning,
                    "Could not read network interfaces from /sys/class/net: {}. Users will need to enter interface names manually.",
                    e
                );
                Ok(Vec::new())
            }
        }
    }

    /// Add an interface to a zone
    pub async fn add_interface_to_zone(
        &self,
        zone_name: &str,
        interface: &str,
    ) -> Result<(), GfwdError> {
        // Validate interface name first
        crate::core::validation::validate_interface_name(interface)?;

        let cfg = gfwd_bus::config_firewalld1::ConfigFirewalld1Proxy::new(&self.conn).await?;
        let path = cfg.get_zone_by_name(zone_name).await?;
        let proxy = gfwd_bus::config_zone::ConfigZoneProxy::builder(&self.conn)
            .path(path.as_str())?
            .build()
            .await?;
        proxy.add_interface(interface).await?;
        Ok(())
    }

    /// Remove an interface from a zone
    pub async fn remove_interface_from_zone(
        &self,
        zone_name: &str,
        interface: &str,
    ) -> Result<(), GfwdError> {
        let cfg = gfwd_bus::config_firewalld1::ConfigFirewalld1Proxy::new(&self.conn).await?;
        let path = cfg.get_zone_by_name(zone_name).await?;
        let proxy = gfwd_bus::config_zone::ConfigZoneProxy::builder(&self.conn)
            .path(path.as_str())?
            .build()
            .await?;
        proxy.remove_interface(interface).await?;
        Ok(())
    }

    /// Add a source address to a zone
    pub async fn add_source_to_zone(&self, zone_name: &str, source: &str) -> Result<(), GfwdError> {
        // Validate source address first
        crate::core::validation::validate_source_address(source)?;

        let cfg = gfwd_bus::config_firewalld1::ConfigFirewalld1Proxy::new(&self.conn).await?;
        let path = cfg.get_zone_by_name(zone_name).await?;
        let proxy = gfwd_bus::config_zone::ConfigZoneProxy::builder(&self.conn)
            .path(path.as_str())?
            .build()
            .await?;
        proxy.add_source(source).await?;
        Ok(())
    }

    /// Remove a source address from a zone
    pub async fn remove_source_from_zone(
        &self,
        zone_name: &str,
        source: &str,
    ) -> Result<(), GfwdError> {
        let cfg = gfwd_bus::config_firewalld1::ConfigFirewalld1Proxy::new(&self.conn).await?;
        let path = cfg.get_zone_by_name(zone_name).await?;
        let proxy = gfwd_bus::config_zone::ConfigZoneProxy::builder(&self.conn)
            .path(path.as_str())?
            .build()
            .await?;
        proxy.remove_source(source).await?;
        Ok(())
    }

    /// Get all IP sets
    pub async fn get_ipsets(&self) -> Result<Vec<String>, GfwdError> {
        let cfg = gfwd_bus::config_firewalld1::ConfigFirewalld1Proxy::new(&self.conn).await?;
        let mut ipsets = cfg.get_ipset_names().await?;
        ipsets.sort();
        Ok(ipsets)
    }

    /// Create a new IP set
    pub async fn create_ipset(&self, settings: IPSetSettings) -> Result<(), GfwdError> {
        // Validate IP set name
        crate::core::validation::validate_ipset_name(&settings.name)?;
        
        // Validate IP set type
        crate::core::validation::validate_ipset_type(&settings.ipset_type)?;

        // Validate entries
        for entry in &settings.entries {
            crate::core::validation::validate_ipset_entry(entry, &settings.ipset_type)?;
        }

        let cfg = gfwd_bus::config_firewalld1::ConfigFirewalld1Proxy::new(&self.conn).await?;
        let ipset_settings: gfwd_bus::config_firewalld1::IPSetSettings = (
            "1.0".to_string(),      // version
            settings.name.clone(),   // name
            "".to_string(),         // description (empty by default)
            settings.ipset_type,    // type
            settings.options,       // options
            settings.entries,       // entries
        );
        
        cfg.add_ipset(&settings.name, &ipset_settings).await?;
        Ok(())
    }

    /// Delete an IP set
    pub async fn delete_ipset(&self, ipset_name: &str) -> Result<(), GfwdError> {
        let cfg = gfwd_bus::config_firewalld1::ConfigFirewalld1Proxy::new(&self.conn).await?;
        let path = cfg.get_ipset_by_name(ipset_name).await?;
        let proxy = gfwd_bus::config_ipset::ConfigIPSetProxy::builder(&self.conn)
            .path(path.as_str())?
            .build()
            .await?;
        proxy.remove().await?;
        Ok(())
    }

    /// Get entries for a specific IP set
    #[allow(dead_code)]
    pub async fn get_ipset_entries(&self, ipset_name: &str) -> Result<Vec<String>, GfwdError> {
        let cfg = gfwd_bus::config_firewalld1::ConfigFirewalld1Proxy::new(&self.conn).await?;
        let path = cfg.get_ipset_by_name(ipset_name).await?;
        let proxy = gfwd_bus::config_ipset::ConfigIPSetProxy::builder(&self.conn)
            .path(path.as_str())?
            .build()
            .await?;
        let entries = proxy.get_entries().await?;
        Ok(entries)
    }

    /// Add an entry to an IP set
    #[allow(dead_code)]
    pub async fn add_ipset_entry(&self, ipset_name: &str, entry: &str) -> Result<(), GfwdError> {
        // Get IP set type for validation
        let ipset_type = self.get_ipset_type(ipset_name).await?;
        
        // Validate entry
        crate::core::validation::validate_ipset_entry(entry, &ipset_type)?;

        let cfg = gfwd_bus::config_firewalld1::ConfigFirewalld1Proxy::new(&self.conn).await?;
        let path = cfg.get_ipset_by_name(ipset_name).await?;
        let proxy = gfwd_bus::config_ipset::ConfigIPSetProxy::builder(&self.conn)
            .path(path.as_str())?
            .build()
            .await?;
        proxy.add_entry(entry).await?;
        Ok(())
    }

    /// Remove an entry from an IP set
    #[allow(dead_code)]
    pub async fn remove_ipset_entry(&self, ipset_name: &str, entry: &str) -> Result<(), GfwdError> {
        let cfg = gfwd_bus::config_firewalld1::ConfigFirewalld1Proxy::new(&self.conn).await?;
        let path = cfg.get_ipset_by_name(ipset_name).await?;
        let proxy = gfwd_bus::config_ipset::ConfigIPSetProxy::builder(&self.conn)
            .path(path.as_str())?
            .build()
            .await?;
        proxy.remove_entry(entry).await?;
        Ok(())
    }

    /// Get IP set type (helper method for validation)
    #[allow(dead_code)]
    async fn get_ipset_type(&self, ipset_name: &str) -> Result<String, GfwdError> {
        let cfg = gfwd_bus::config_firewalld1::ConfigFirewalld1Proxy::new(&self.conn).await?;
        let path = cfg.get_ipset_by_name(ipset_name).await?;
        let proxy = gfwd_bus::config_ipset::ConfigIPSetProxy::builder(&self.conn)
            .path(path.as_str())?
            .build()
            .await?;
        let ipset_type = proxy.get_type().await?;
        Ok(ipset_type)
    }

    /// Get all rich rules for a zone
    pub async fn get_rich_rules(&self, zone_name: &str) -> Result<Vec<String>, GfwdError> {
        let cfg = gfwd_bus::config_firewalld1::ConfigFirewalld1Proxy::new(&self.conn).await?;
        let path = cfg.get_zone_by_name(zone_name).await?;
        let proxy = gfwd_bus::config_zone::ConfigZoneProxy::builder(&self.conn)
            .path(path.as_str())?
            .build()
            .await?;
        let rich_rules = proxy.get_rich_rules().await?;
        Ok(rich_rules)
    }

    /// Add a rich rule to a zone
    pub async fn add_rich_rule(&self, zone_name: &str, rule_xml: &str) -> Result<(), GfwdError> {
        // Validate the rich rule XML
        let validated_xml = crate::core::validation::validate_rich_rule_xml(rule_xml)?;

        let cfg = gfwd_bus::config_firewalld1::ConfigFirewalld1Proxy::new(&self.conn).await?;
        let path = cfg.get_zone_by_name(zone_name).await?;
        let proxy = gfwd_bus::config_zone::ConfigZoneProxy::builder(&self.conn)
            .path(path.as_str())?
            .build()
            .await?;
        proxy.add_rich_rule(&validated_xml).await?;
        Ok(())
    }

    /// Remove a rich rule from a zone
    pub async fn remove_rich_rule(&self, zone_name: &str, rule_xml: &str) -> Result<(), GfwdError> {
        let cfg = gfwd_bus::config_firewalld1::ConfigFirewalld1Proxy::new(&self.conn).await?;
        let path = cfg.get_zone_by_name(zone_name).await?;
        let proxy = gfwd_bus::config_zone::ConfigZoneProxy::builder(&self.conn)
            .path(path.as_str())?
            .build()
            .await?;
        proxy.remove_rich_rule(rule_xml).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;
    use crate::models::rich_rule::{RichRule, RichRuleAction};

    #[test]
    fn test_rich_rule_xml_generation() {
        // Test basic accept rule
        let rule = RichRule::new().with_action(RichRuleAction::Accept);
        let xml = rule.to_xml();
        assert_eq!(xml, "<rule><accept/></rule>");

        // Test rule with source and service
        let rule = RichRule::new()
            .with_source("192.168.1.0/24".to_string(), false)
            .with_service("ssh".to_string())
            .with_action(RichRuleAction::Accept);
        let xml = rule.to_xml();
        assert!(xml.contains("<source address=\"192.168.1.0/24\"/>"));
        assert!(xml.contains("<service name=\"ssh\"/>"));
        assert!(xml.contains("<accept/>"));

        // Test rule with port and reject action
        let rule = RichRule::new()
            .with_port("80".to_string(), "tcp".to_string())
            .with_action(RichRuleAction::Reject(Some("icmp-port-unreachable".to_string())));
        let xml = rule.to_xml();
        assert!(xml.contains("<port port=\"80\" protocol=\"tcp\"/>"));
        assert!(xml.contains("<reject type=\"icmp-port-unreachable\"/>"));

        // Test rule with family and destination
        let rule = RichRule::new()
            .with_family("ipv4".to_string())
            .with_destination("10.0.0.1".to_string(), true)
            .with_action(RichRuleAction::Drop);
        let xml = rule.to_xml();
        assert!(xml.contains("family=\"ipv4\""));
        assert!(xml.contains("<destination invert=\"true\" address=\"10.0.0.1\"/>"));
        assert!(xml.contains("<drop/>"));

        // Test rule with mark action
        let rule = RichRule::new()
            .with_protocol("icmp".to_string())
            .with_action(RichRuleAction::Mark("0x1".to_string()));
        let xml = rule.to_xml();
        assert!(xml.contains("<protocol value=\"icmp\"/>"));
        assert!(xml.contains("<mark set=\"0x1\"/>"));
    }

    #[test]
    fn test_rich_rule_validation_integration() {
        // Test that generated XML passes validation
        let rule = RichRule::new()
            .with_source("192.168.1.0/24".to_string(), false)
            .with_service("ssh".to_string())
            .with_action(RichRuleAction::Accept);
        let xml = rule.to_xml();
        
        // This should pass validation
        assert!(crate::core::validation::validate_rich_rule_xml(&xml).is_ok());

        // Test invalid XML fails validation
        assert!(crate::core::validation::validate_rich_rule_xml("<rule>no action</rule>").is_err());
        assert!(crate::core::validation::validate_rich_rule_xml("not xml").is_err());
    }

    #[test]
    fn test_rich_rule_action_variants() {
        // Test all action variants
        let accept_rule = RichRule::new().with_action(RichRuleAction::Accept);
        assert!(accept_rule.to_xml().contains("<accept/>"));

        let reject_rule = RichRule::new().with_action(RichRuleAction::Reject(None));
        assert!(reject_rule.to_xml().contains("<reject/>"));

        let reject_with_type = RichRule::new().with_action(RichRuleAction::Reject(Some("icmp-host-prohibited".to_string())));
        assert!(reject_with_type.to_xml().contains("<reject type=\"icmp-host-prohibited\"/>"));

        let drop_rule = RichRule::new().with_action(RichRuleAction::Drop);
        assert!(drop_rule.to_xml().contains("<drop/>"));

        let mark_rule = RichRule::new().with_action(RichRuleAction::Mark("0x2".to_string()));
        assert!(mark_rule.to_xml().contains("<mark set=\"0x2\"/>"));
    }

    #[test]
    fn test_rich_rule_builder_pattern() {
        // Test that builder pattern works correctly
        let rule = RichRule::new()
            .with_family("ipv6".to_string())
            .with_source("2001:db8::/32".to_string(), false)
            .with_destination("::1".to_string(), true)
            .with_service("http".to_string())
            .with_action(RichRuleAction::Accept);

        assert_eq!(rule.family, Some("ipv6".to_string()));
        assert!(rule.source.is_some());
        assert!(rule.destination.is_some());
        assert_eq!(rule.service, Some("http".to_string()));
        assert_eq!(rule.action, RichRuleAction::Accept);

        // Test that destination has invert flag set
        if let Some(dest) = &rule.destination {
            assert!(dest.invert);
            assert_eq!(dest.address, "::1");
        }
    }
}
