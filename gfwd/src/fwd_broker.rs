use std::collections::HashMap;

use gfwd_bus::config_firewalld1::ZoneSettings as ZoneSettingsBus;
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

#[derive(Debug, Default)]
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

#[derive(Debug, Default, derive_more::Display)]
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

    // pub async fn remove_zone(&self, name: &str) -> Result<(), GfwdError> {
    //     // Ok(self.cfg_fwd.remove_zone(name).await?)
    //     Ok(())
    // }

    // pub async fn list_services(&self) -> Result<Vec<String>, GfwdError> {
    //     Ok(self.fwd.list_services().await?)
    // }
}
