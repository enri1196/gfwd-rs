use std::collections::HashMap;

use gfwd_bus::{
    config_firewalld1::ZoneSettings as ZoneSettingsBus,
    config_zone::new_config_zone_proxy,
    config_firewalld1::new_config_firewalld1_proxy
};
use relm4::tokio::sync::OnceCell;

use crate::error::GfwdError;

pub struct FwdBroker {
    fwd_zone: gfwd_bus::zone::ZoneProxy<'static>,
    cfg_fwd: gfwd_bus::config_firewalld1::ConfigFirewalld1Proxy<'static>,
}

impl PartialEq for FwdBroker {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct ZoneSettings {
    pub version: String,
    pub name: String,
    pub description: String,
    pub unused: bool,
    pub target: ZoneTarget,
    pub services: Vec<String>,
    pub ports: Vec<(String, String)>,
    pub icmp_blocks: Vec<String>,
    pub masquerade: bool,
    pub forward_ports: Vec<(String, String, String, String)>,
    pub interfaces: Vec<String>,
    pub sources: Vec<String>,
    pub rich_rules: Vec<String>,
    pub protocols: Vec<String>,
    pub source_ports: Vec<(String, String)>,
}

#[derive(Debug, Default, derive_more::Display, Clone, PartialEq)]
#[allow(unused)]
pub enum ZoneTarget {
    #[default]
    #[display("default")]
    Default,
    #[display("ACCEPT")]
    Accept,
    #[display("DROP")]
    Drop,
    #[display("REJECT")]
    Reject,
}

// Static object holding the sender side of the channel
static BROKER: OnceCell<FwdBroker> = OnceCell::const_new();

impl FwdBroker {
    pub async fn get_broker() -> &'static FwdBroker {
        BROKER
            .get_or_init(|| async move {
                FwdBroker {
                    fwd_zone: gfwd_bus::zone::new_zone_proxy().await.unwrap(),
                    cfg_fwd: gfwd_bus::config_firewalld1::new_config_firewalld1_proxy()
                        .await
                        .unwrap(),
                }
            })
            .await
    }

    /// Get all zones
    pub async fn get_zones(&self) -> Result<Vec<String>, GfwdError> {
        Ok(self.cfg_fwd.get_zone_names().await?)
    }

    pub async fn get_active_zones(
        &self,
    ) -> Result<HashMap<String, HashMap<String, Vec<String>>>, GfwdError> {
        Ok(self.fwd_zone.get_active_zones().await?)
    }

    /// Get the default zone
    pub async fn get_default_zone(&self) -> Result<String, GfwdError> {
        Ok(self.cfg_fwd.default_zone().await?)
    }

    pub async fn get_zone_settings(&self, zone_name: &str) -> Result<ZoneSettings, GfwdError> {
        let cfg_proxy = new_config_firewalld1_proxy().await?;
        let proxy = new_config_zone_proxy(&cfg_proxy, zone_name).await?;

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
        Ok(self
            .cfg_fwd
            .add_zone(name.as_str(), &zone_settings)
            .await
            .map(|_| ())?)
    }
}
