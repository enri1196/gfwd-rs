//! Permanent and runtime zone loading, snapshots, and mutations.

use std::collections::HashSet;

use gfwd_bus::config_zone::ConfigZoneProxy;
use gfwd_bus::zone::ZoneProxy;

use crate::core::reconciliation::{ZoneReconciliationData, ZoneSettingsSnapshot};
use crate::models::{ZoneDetails, ZoneTarget};

use super::{BrokerError, FwdBroker};

impl FwdBroker {
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

    /// List permanent zone names in stable order.
    pub async fn get_zones(&self) -> Result<Vec<String>, BrokerError> {
        let cfg = self.config().await?;
        let mut zones = cfg.get_zone_names().await?;
        zones.sort();
        Ok(zones)
    }

    /// Permanently enable a configured service in a zone.
    pub async fn add_service(&self, zone_name: &str, service: &str) -> Result<(), BrokerError> {
        self.zone(zone_name).await?.add_service(service).await?;
        Ok(())
    }

    /// Return the set of zones active at runtime.
    pub async fn get_active_zones(&self) -> Result<HashSet<String>, BrokerError> {
        let proxy = ZoneProxy::new(&self.conn).await?;
        let active = proxy.get_active_zones().await?;
        Ok(active.into_keys().collect())
    }

    /// Create a permanent zone.
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

    /// Permanently add a port to a zone.
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

    /// Permanently add a forwarded port to a zone.
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

    /// Permanently bind an interface to a zone.
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

    /// Permanently add a source to a zone.
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

    /// Permanently add an ICMP block to a zone.
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

    /// Permanently add a rich rule to a zone.
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

    /// Permanently delete a zone.
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

    /// Permanently remove a service from a zone.
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

    /// Permanently unbind an interface from a zone.
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

    /// Permanently remove a source from a zone.
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

    /// Permanently remove a port from a zone.
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

    /// Permanently remove a forwarded port from a zone.
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

    /// Permanently remove a source port from a zone.
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

    /// Permanently remove an ICMP block from a zone.
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

    /// Permanently remove a rich rule from a zone.
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

    /// Load permanent details for a zone.
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
}
