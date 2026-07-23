use gfwd_bus::config_firewalld1::ConfigFirewalld1Proxy;
use gfwd_bus::config_icmptype::ConfigIcmpTypeProxy;
use gfwd_bus::config_ipset::ConfigIPSetProxy;
use gfwd_bus::config_zone::ConfigZoneProxy;
use gfwd_bus::firewalld1::FirewallD1Proxy;
use gfwd_bus::network_manager::{DeviceProxy, NetworkManagerProxy};
use gfwd_bus::systemd::{ManagerProxy, UnitProxy};
use gfwd_bus::zone::ZoneProxy;
use tokio::sync::OnceCell;
use zbus::Connection;

use super::reconciliation::{ZoneReconciliationData, ZoneSettingsParseError, ZoneSettingsSnapshot};
use std::collections::HashSet;

use crate::models::{IcmpTypeInfo, IpSetDetails, ZoneDetails, ZoneTarget};

#[derive(Clone, Debug)]
/// Error returned by broker-owned system-bus operations.
pub struct BrokerError {
    message: String,
}

/// Current activation state of `firewalld.service`.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum FirewalldStatus {
    /// The unit state is being queried or is transitioning.
    #[default]
    Loading,
    /// The systemd unit is active.
    Active,
    /// The systemd unit is not active.
    Inactive,
    /// The unit state could not be queried.
    Error(String),
}

impl BrokerError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for BrokerError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for BrokerError {}

impl From<zbus::Error> for BrokerError {
    fn from(error: zbus::Error) -> Self {
        Self::new(error.to_string())
    }
}

impl From<ZoneSettingsParseError> for BrokerError {
    fn from(error: ZoneSettingsParseError) -> Self {
        Self::new(error.to_string())
    }
}

#[derive(Debug)]
/// Shared owner of all firewalld, systemd, and NetworkManager D-Bus proxies.
pub struct FwdBroker {
    conn: Connection,
}

static BROKER: OnceCell<FwdBroker> = OnceCell::const_new();

impl FwdBroker {
    /// Returns the lazily initialized system-bus broker.
    pub async fn get() -> Result<&'static FwdBroker, BrokerError> {
        BROKER
            .get_or_try_init(|| async {
                let conn = Connection::system().await.map_err(BrokerError::from)?;
                Ok(FwdBroker { conn })
            })
            .await
    }

    async fn config(&self) -> Result<ConfigFirewalld1Proxy<'_>, BrokerError> {
        Ok(ConfigFirewalld1Proxy::new(&self.conn).await?)
    }

    async fn zone(&self, zone_name: &str) -> Result<ConfigZoneProxy<'_>, BrokerError> {
        let cfg = self.config().await?;
        let path = cfg.get_zone_by_name(zone_name).await?;
        Ok(ConfigZoneProxy::builder(&self.conn)
            .path(path)?
            .build()
            .await?)
    }

    /// Load and decode every known permanent setting for a zone.
    pub async fn permanent_zone_snapshot(
        &self,
        zone_name: &str,
    ) -> Result<ZoneSettingsSnapshot, BrokerError> {
        let settings = self.zone(zone_name).await?.get_settings2().await?;
        Ok(ZoneSettingsSnapshot::from_settings(settings)?)
    }

    /// Load and decode every known runtime setting for a zone.
    pub async fn runtime_zone_snapshot(
        &self,
        zone_name: &str,
    ) -> Result<ZoneSettingsSnapshot, BrokerError> {
        let settings = ZoneProxy::new(&self.conn)
            .await?
            .get_settings2(zone_name)
            .await?;
        Ok(ZoneSettingsSnapshot::from_settings(settings)?)
    }

    /// Load both selected-zone snapshots and compute their pure reconciliation.
    pub async fn reconcile_zone(
        &self,
        zone_name: &str,
    ) -> Result<ZoneReconciliationData, BrokerError> {
        let permanent = self.permanent_zone_snapshot(zone_name).await?;
        let runtime = self.runtime_zone_snapshot(zone_name).await?;
        Ok(ZoneReconciliationData::new(permanent, runtime))
    }

    /// Permanently enables or disables masquerading for a zone.
    pub async fn set_masquerade(&self, zone_name: &str, enabled: bool) -> Result<(), BrokerError> {
        let proxy = self.zone(zone_name).await?;
        if enabled {
            proxy.add_masquerade().await?;
        } else {
            proxy.remove_masquerade().await?;
        }
        Ok(())
    }

    /// Permanently sets ICMP block inversion for a zone.
    pub async fn set_icmp_block_inversion(
        &self,
        zone_name: &str,
        enabled: bool,
    ) -> Result<(), BrokerError> {
        self.zone(zone_name)
            .await?
            .set_icmp_block_inversion(enabled)
            .await?;
        Ok(())
    }

    /// Returns the current systemd activation state of `firewalld.service`.
    pub async fn firewalld_status(&self) -> Result<FirewalldStatus, BrokerError> {
        let manager = ManagerProxy::new(&self.conn).await?;
        let path = manager.get_unit("firewalld.service").await?;
        let unit = UnitProxy::builder(&self.conn)
            .path(path.as_str())?
            .build()
            .await?;
        Ok(match unit.active_state().await?.as_str() {
            "active" => FirewalldStatus::Active,
            "activating" | "deactivating" | "reloading" => FirewalldStatus::Loading,
            _ => FirewalldStatus::Inactive,
        })
    }

    /// Requests systemd to start `firewalld.service`.
    pub async fn start_firewalld(&self) -> Result<(), BrokerError> {
        ManagerProxy::new(&self.conn)
            .await?
            .start_unit("firewalld.service", "replace")
            .await?;
        Ok(())
    }

    /// Requests systemd to stop `firewalld.service`.
    pub async fn stop_firewalld(&self) -> Result<(), BrokerError> {
        ManagerProxy::new(&self.conn)
            .await?
            .stop_unit("firewalld.service", "replace")
            .await?;
        Ok(())
    }

    /// Reloads firewalld so permanent configuration becomes active at runtime.
    pub async fn apply_permanent_configuration(&self) -> Result<(), BrokerError> {
        FirewallD1Proxy::new(&self.conn).await?.reload().await?;
        Ok(())
    }

    /// Persist all current runtime firewalld configuration globally.
    pub async fn persist_runtime_configuration(&self) -> Result<(), BrokerError> {
        FirewallD1Proxy::new(&self.conn)
            .await?
            .runtime_to_permanent()
            .await?;
        Ok(())
    }

    pub async fn get_zones(&self) -> Result<Vec<String>, BrokerError> {
        let cfg = self.config().await?;
        let mut zones = cfg.get_zone_names().await?;
        zones.sort();
        Ok(zones)
    }

    /// Lists the permanent firewalld service catalog in name order.
    pub async fn get_services(&self) -> Result<Vec<String>, BrokerError> {
        let mut services = self.config().await?.get_service_names().await?;
        services.sort();
        Ok(services)
    }

    /// Lists configured ICMP types and descriptions in name order.
    pub async fn get_icmp_types(&self) -> Result<Vec<IcmpTypeInfo>, BrokerError> {
        let cfg = self.config().await?;
        let names = cfg.get_icmp_type_names().await?;
        let mut types = Vec::with_capacity(names.len());
        for name in names {
            let path = cfg.get_icmp_type_by_name(&name).await?;
            let proxy = ConfigIcmpTypeProxy::builder(&self.conn)
                .path(path)?
                .build()
                .await?;
            types.push(IcmpTypeInfo {
                description: proxy.get_description().await?,
                name,
            });
        }
        types.sort_by(|left, right| left.name.cmp(&right.name));
        Ok(types)
    }

    /// Permanently enables a configured service in a zone.
    pub async fn add_service(&self, zone_name: &str, service: &str) -> Result<(), BrokerError> {
        self.zone(zone_name).await?.add_service(service).await?;
        Ok(())
    }

    pub async fn get_default_zone(&self) -> Result<String, BrokerError> {
        let cfg = self.config().await?;
        Ok(cfg.default_zone().await?)
    }

    pub async fn set_default_zone(&self, zone_name: &str) -> Result<(), BrokerError> {
        let proxy = FirewallD1Proxy::new(&self.conn).await?;
        proxy.set_default_zone(zone_name).await?;
        Ok(())
    }

    pub async fn get_active_zones(&self) -> Result<HashSet<String>, BrokerError> {
        let proxy = ZoneProxy::new(&self.conn).await?;
        let active = proxy.get_active_zones().await?;
        Ok(active.into_keys().collect())
    }

    pub async fn get_interfaces(&self) -> Result<Vec<String>, BrokerError> {
        match self.get_interfaces_from_networkmanager().await {
            Ok(interfaces) if !interfaces.is_empty() => Ok(interfaces),
            Ok(_) | Err(_) => self.get_interfaces_from_sysfs().await,
        }
    }

    async fn get_interfaces_from_networkmanager(&self) -> Result<Vec<String>, BrokerError> {
        let proxy = NetworkManagerProxy::new(&self.conn).await?;
        let devices = proxy.get_devices().await?;
        let mut interfaces = Vec::with_capacity(devices.len());

        for path in devices {
            let device = DeviceProxy::builder(&self.conn)
                .path(path.as_str())?
                .build()
                .await?;
            let name = device.interface().await?;
            if should_include_interface(&name) {
                interfaces.push(name);
            }
        }

        interfaces.sort();
        interfaces.dedup();
        Ok(interfaces)
    }

    async fn get_interfaces_from_sysfs(&self) -> Result<Vec<String>, BrokerError> {
        use std::fs;

        let entries = match fs::read_dir("/sys/class/net") {
            Ok(entries) => entries,
            Err(error) => {
                eprintln!("failed to read /sys/class/net: {error}");
                return Ok(Vec::new());
            }
        };

        let mut interfaces = Vec::new();
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str()
                && should_include_interface(name)
            {
                interfaces.push(name.to_string());
            }
        }

        interfaces.sort();
        interfaces.dedup();
        Ok(interfaces)
    }

    pub async fn add_zone(
        &self,
        name: &str,
        description: &str,
        target: &ZoneTarget,
    ) -> Result<(), BrokerError> {
        let cfg = self.config().await?;
        let settings: gfwd_bus::config_firewalld1::ZoneSettings = (
            "1.0".to_string(),
            name.to_string(),
            description.to_string(),
            false,
            target.to_string(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            false,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
        );
        cfg.add_zone(name, &settings).await?;
        Ok(())
    }

    pub async fn add_port(
        &self,
        zone_name: &str,
        port: &str,
        protocol: &str,
    ) -> Result<(), BrokerError> {
        let cfg = self.config().await?;
        let path = cfg.get_zone_by_name(zone_name).await?;
        let proxy = ConfigZoneProxy::builder(&self.conn)
            .path(path.as_str())?
            .build()
            .await?;
        proxy.add_port(port, protocol).await?;
        Ok(())
    }

    pub async fn add_forward_port(
        &self,
        zone_name: &str,
        port: &str,
        protocol: &str,
        to_port: &str,
        to_addr: &str,
    ) -> Result<(), BrokerError> {
        let cfg = self.config().await?;
        let path = cfg.get_zone_by_name(zone_name).await?;
        let proxy = ConfigZoneProxy::builder(&self.conn)
            .path(path.as_str())?
            .build()
            .await?;
        proxy
            .add_forward_port(port, protocol, to_port, to_addr)
            .await?;
        Ok(())
    }

    pub async fn add_interface(&self, zone_name: &str, interface: &str) -> Result<(), BrokerError> {
        let cfg = self.config().await?;
        let path = cfg.get_zone_by_name(zone_name).await?;
        let proxy = ConfigZoneProxy::builder(&self.conn)
            .path(path.as_str())?
            .build()
            .await?;
        proxy.add_interface(interface).await?;
        Ok(())
    }

    pub async fn add_source(&self, zone_name: &str, source: &str) -> Result<(), BrokerError> {
        let cfg = self.config().await?;
        let path = cfg.get_zone_by_name(zone_name).await?;
        let proxy = ConfigZoneProxy::builder(&self.conn)
            .path(path.as_str())?
            .build()
            .await?;
        proxy.add_source(source).await?;
        Ok(())
    }

    pub async fn add_icmp_block(&self, zone_name: &str, icmp: &str) -> Result<(), BrokerError> {
        let cfg = self.config().await?;
        let path = cfg.get_zone_by_name(zone_name).await?;
        let proxy = ConfigZoneProxy::builder(&self.conn)
            .path(path.as_str())?
            .build()
            .await?;
        proxy.add_icmp_block(icmp).await?;
        Ok(())
    }

    pub async fn add_rich_rule(&self, zone_name: &str, rule: &str) -> Result<(), BrokerError> {
        let cfg = self.config().await?;
        let path = cfg.get_zone_by_name(zone_name).await?;
        let proxy = ConfigZoneProxy::builder(&self.conn)
            .path(path.as_str())?
            .build()
            .await?;
        proxy.add_rich_rule(rule).await?;
        Ok(())
    }

    pub async fn remove_zone(&self, zone_name: &str) -> Result<(), BrokerError> {
        let cfg = self.config().await?;
        let path = cfg.get_zone_by_name(zone_name).await?;
        let proxy = ConfigZoneProxy::builder(&self.conn)
            .path(path.as_str())?
            .build()
            .await?;
        proxy.remove().await?;
        Ok(())
    }
    pub async fn remove_service(&self, zone_name: &str, service: &str) -> Result<(), BrokerError> {
        let cfg = self.config().await?;
        let path = cfg.get_zone_by_name(zone_name).await?;
        let proxy = ConfigZoneProxy::builder(&self.conn)
            .path(path.as_str())?
            .build()
            .await?;
        proxy.remove_service(service).await?;
        Ok(())
    }

    pub async fn remove_interface(
        &self,
        zone_name: &str,
        interface: &str,
    ) -> Result<(), BrokerError> {
        let cfg = self.config().await?;
        let path = cfg.get_zone_by_name(zone_name).await?;
        let proxy = ConfigZoneProxy::builder(&self.conn)
            .path(path.as_str())?
            .build()
            .await?;
        proxy.remove_interface(interface).await?;
        Ok(())
    }

    pub async fn remove_source(&self, zone_name: &str, source: &str) -> Result<(), BrokerError> {
        let cfg = self.config().await?;
        let path = cfg.get_zone_by_name(zone_name).await?;
        let proxy = ConfigZoneProxy::builder(&self.conn)
            .path(path.as_str())?
            .build()
            .await?;
        proxy.remove_source(source).await?;
        Ok(())
    }

    pub async fn remove_port(
        &self,
        zone_name: &str,
        port: &str,
        protocol: &str,
    ) -> Result<(), BrokerError> {
        let cfg = self.config().await?;
        let path = cfg.get_zone_by_name(zone_name).await?;
        let proxy = ConfigZoneProxy::builder(&self.conn)
            .path(path.as_str())?
            .build()
            .await?;
        proxy.remove_port(port, protocol).await?;
        Ok(())
    }

    pub async fn remove_forward_port(
        &self,
        zone_name: &str,
        port: &str,
        protocol: &str,
        to_port: &str,
        to_addr: &str,
    ) -> Result<(), BrokerError> {
        let cfg = self.config().await?;
        let path = cfg.get_zone_by_name(zone_name).await?;
        let proxy = ConfigZoneProxy::builder(&self.conn)
            .path(path.as_str())?
            .build()
            .await?;
        proxy
            .remove_forward_port(port, protocol, to_port, to_addr)
            .await?;
        Ok(())
    }

    pub async fn remove_source_port(
        &self,
        zone_name: &str,
        port: &str,
        protocol: &str,
    ) -> Result<(), BrokerError> {
        let cfg = self.config().await?;
        let path = cfg.get_zone_by_name(zone_name).await?;
        let proxy = ConfigZoneProxy::builder(&self.conn)
            .path(path.as_str())?
            .build()
            .await?;
        proxy.remove_source_port(port, protocol).await?;
        Ok(())
    }

    pub async fn remove_icmp_block(&self, zone_name: &str, icmp: &str) -> Result<(), BrokerError> {
        let cfg = self.config().await?;
        let path = cfg.get_zone_by_name(zone_name).await?;
        let proxy = ConfigZoneProxy::builder(&self.conn)
            .path(path.as_str())?
            .build()
            .await?;
        proxy.remove_icmp_block(icmp).await?;
        Ok(())
    }

    pub async fn remove_rich_rule(&self, zone_name: &str, rule: &str) -> Result<(), BrokerError> {
        let cfg = self.config().await?;
        let path = cfg.get_zone_by_name(zone_name).await?;
        let proxy = ConfigZoneProxy::builder(&self.conn)
            .path(path.as_str())?
            .build()
            .await?;
        proxy.remove_rich_rule(rule).await?;
        Ok(())
    }

    pub async fn get_zone_details(&self, zone_name: &str) -> Result<ZoneDetails, BrokerError> {
        let cfg = self.config().await?;
        let path = cfg.get_zone_by_name(zone_name).await?;
        let proxy = ConfigZoneProxy::builder(&self.conn)
            .path(path.as_str())?
            .build()
            .await?;

        let description = proxy.get_description().await?;
        let target = proxy.get_target().await?;
        let masquerade = proxy.get_masquerade().await?;
        let icmp_block_inversion = proxy.get_icmp_block_inversion().await?;

        let mut services = proxy.get_services().await?;
        services.sort();

        let mut ports = proxy.get_ports().await?;
        ports.sort();

        let mut forward_ports = proxy.get_forward_ports().await?;
        forward_ports.sort();

        let mut interfaces = proxy.get_interfaces().await?;
        interfaces.sort();

        let mut sources = proxy.get_sources().await?;
        sources.sort();

        let mut icmp_blocks = proxy.get_icmp_blocks().await?;
        icmp_blocks.sort();

        let mut rich_rules = proxy.get_rich_rules().await?;
        rich_rules.sort();

        let mut protocols = proxy.get_protocols().await?;
        protocols.sort();

        let mut source_ports = proxy.get_source_ports().await?;
        source_ports.sort();

        Ok(ZoneDetails {
            name: zone_name.to_string(),
            description,
            target: ZoneTarget::from_raw(target),
            masquerade,
            icmp_block_inversion,
            services,
            ports,
            forward_ports,
            interfaces,
            sources,
            icmp_blocks,
            rich_rules,
            protocols,
            source_ports,
        })
    }

    pub async fn get_ipsets(&self) -> Result<Vec<String>, BrokerError> {
        let cfg = self.config().await?;
        let mut ipsets = cfg.get_ipset_names().await?;
        ipsets.sort();
        Ok(ipsets)
    }

    pub async fn get_ipset_details(&self, ipset_name: &str) -> Result<IpSetDetails, BrokerError> {
        let cfg = self.config().await?;
        let path = cfg.get_ipset_by_name(ipset_name).await?;
        let proxy = ConfigIPSetProxy::builder(&self.conn)
            .path(path.as_str())?
            .build()
            .await?;

        let (_version, name, _description, ipset_type, options, mut entries) =
            proxy.get_settings().await?;
        entries.sort();

        Ok(IpSetDetails {
            name,
            ipset_type,
            entries,
            options,
        })
    }

    pub async fn add_ipset_entry(&self, ipset_name: &str, entry: &str) -> Result<(), BrokerError> {
        let cfg = self.config().await?;
        let path = cfg.get_ipset_by_name(ipset_name).await?;
        let proxy = ConfigIPSetProxy::builder(&self.conn)
            .path(path.as_str())?
            .build()
            .await?;
        proxy.add_entry(entry).await?;
        Ok(())
    }
    pub async fn remove_ipset_entry(
        &self,
        ipset_name: &str,
        entry: &str,
    ) -> Result<(), BrokerError> {
        let cfg = self.config().await?;
        let path = cfg.get_ipset_by_name(ipset_name).await?;
        let proxy = ConfigIPSetProxy::builder(&self.conn)
            .path(path.as_str())?
            .build()
            .await?;
        proxy.remove_entry(entry).await?;
        Ok(())
    }

    /// Permanently deletes an IP set.
    pub async fn remove_ipset(&self, ipset_name: &str) -> Result<(), BrokerError> {
        let cfg = self.config().await?;
        let path = cfg.get_ipset_by_name(ipset_name).await?;
        ConfigIPSetProxy::builder(&self.conn)
            .path(path)?
            .build()
            .await?
            .remove()
            .await?;
        Ok(())
    }

    pub async fn create_ipset(
        &self,
        name: &str,
        ipset_type: &str,
        entries: Vec<String>,
    ) -> Result<(), BrokerError> {
        let cfg = self.config().await?;
        let settings: gfwd_bus::config_firewalld1::IPSetSettings = (
            "1.0".to_string(),
            name.to_string(),
            "".to_string(),
            ipset_type.to_string(),
            std::collections::HashMap::new(),
            entries,
        );

        cfg.add_ipset(name, &settings).await?;
        Ok(())
    }
}

fn should_include_interface(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }

    !(name == "lo"
        || name.starts_with("docker")
        || name.starts_with("veth")
        || name.starts_with("br-"))
}
