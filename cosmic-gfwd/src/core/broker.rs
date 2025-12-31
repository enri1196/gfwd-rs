use gfwd_bus::config_firewalld1::ConfigFirewalld1Proxy;
use gfwd_bus::config_ipset::ConfigIPSetProxy;
use gfwd_bus::config_zone::ConfigZoneProxy;
use gfwd_bus::zone::ZoneProxy;
use tokio::sync::OnceCell;
use zbus::Connection;

use std::collections::HashSet;

use crate::models::{IpSetDetails, ZoneDetails, ZoneTarget};

#[derive(Clone, Debug)]
pub struct BrokerError {
    message: String,
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

#[derive(Debug)]
pub struct FwdBroker {
    conn: Connection,
}

static BROKER: OnceCell<FwdBroker> = OnceCell::const_new();

impl FwdBroker {
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

    pub async fn get_zones(&self) -> Result<Vec<String>, BrokerError> {
        let cfg = self.config().await?;
        let mut zones = cfg.get_zone_names().await?;
        zones.sort();
        Ok(zones)
    }

    pub async fn get_default_zone(&self) -> Result<String, BrokerError> {
        let cfg = self.config().await?;
        Ok(cfg.default_zone().await?)
    }

    pub async fn get_active_zones(&self) -> Result<HashSet<String>, BrokerError> {
        let proxy = ZoneProxy::new(&self.conn).await?;
        let active = proxy.get_active_zones().await?;
        Ok(active.into_keys().collect())
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

    pub async fn add_interface(
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
        proxy.add_interface(interface).await?;
        Ok(())
    }

    pub async fn add_source(
        &self,
        zone_name: &str,
        source: &str,
    ) -> Result<(), BrokerError> {
        let cfg = self.config().await?;
        let path = cfg.get_zone_by_name(zone_name).await?;
        let proxy = ConfigZoneProxy::builder(&self.conn)
            .path(path.as_str())?
            .build()
            .await?;
        proxy.add_source(source).await?;
        Ok(())
    }

    pub async fn add_icmp_block(
        &self,
        zone_name: &str,
        icmp: &str,
    ) -> Result<(), BrokerError> {
        let cfg = self.config().await?;
        let path = cfg.get_zone_by_name(zone_name).await?;
        let proxy = ConfigZoneProxy::builder(&self.conn)
            .path(path.as_str())?
            .build()
            .await?;
        proxy.add_icmp_block(icmp).await?;
        Ok(())
    }

    pub async fn add_rich_rule(
        &self,
        zone_name: &str,
        rule: &str,
    ) -> Result<(), BrokerError> {
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
    pub async fn remove_service(
        &self,
        zone_name: &str,
        service: &str,
    ) -> Result<(), BrokerError> {
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

    pub async fn remove_source(
        &self,
        zone_name: &str,
        source: &str,
    ) -> Result<(), BrokerError> {
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

    pub async fn remove_icmp_block(
        &self,
        zone_name: &str,
        icmp: &str,
    ) -> Result<(), BrokerError> {
        let cfg = self.config().await?;
        let path = cfg.get_zone_by_name(zone_name).await?;
        let proxy = ConfigZoneProxy::builder(&self.conn)
            .path(path.as_str())?
            .build()
            .await?;
        proxy.remove_icmp_block(icmp).await?;
        Ok(())
    }

    pub async fn remove_rich_rule(
        &self,
        zone_name: &str,
        rule: &str,
    ) -> Result<(), BrokerError> {
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

    pub async fn add_ipset_entry(
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
