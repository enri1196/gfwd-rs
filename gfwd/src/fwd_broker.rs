use std::io::Result;

use gfwd_bus::config_firewalld1::ZoneSettings as ZoneSettingsBus;
use tokio::sync::OnceCell;

pub struct FwdBroker {
    pub fwd: gfwd_bus::firewalld1::FirewallD1Proxy<'static>,
    pub cfg_fwd: gfwd_bus::config_firewalld1::ConfigFirewalld1Proxy<'static>,
}

#[derive(Debug, Default)]
pub struct ZoneSettings {
    pub version: String,
    pub name: String,
    pub description: String,
    pub unused: bool,
    pub target: String,
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

// Static object holding the sender side of the channel
static BROKER: OnceCell<FwdBroker> = OnceCell::const_new();

impl FwdBroker {
    pub async fn get_broker() -> &'static FwdBroker {
        BROKER.get_or_init(|| async move {
            FwdBroker {
                fwd: gfwd_bus::firewalld1::new_firewalld_proxy().await.unwrap(),
                cfg_fwd: gfwd_bus::config_firewalld1::new_config_firewalld1_proxy().await.unwrap(),
            }
        }).await
    }

    /// Get all zones
    pub async fn get_zones(&self) -> Result<Vec<String>> {
        self.cfg_fwd.get_zone_names().await
            .map_err(|e| std::io::Error::new(get_kind(&e), e))
    }

    /// Get the default zone
    pub async fn get_default_zone(&self) -> Result<String> {
        self.cfg_fwd.default_zone().await
            .map_err(|e| std::io::Error::new(get_kind(&e), e))
    }

    /// Add a new zone with the given settings
    pub async fn add_zone(&self, settings: ZoneSettings) -> Result<()> {
        let name = settings.name.clone();
        let zone_settings: ZoneSettingsBus = (
            settings.version, // version
            settings.name, // name
            settings.description, // description
            settings.unused, // UNUSED
            settings.target, // target
            settings.services, // services
            settings.ports, // ports
            settings.icmp_blocks, // icmp-blocks
            settings.masquerade, // masquerade
            settings.forward_ports, // forward-ports
            settings.interfaces, // interfaces
            settings.sources,
            settings.rich_rules,
            settings.protocols,
            settings.source_ports,
        );
        self.cfg_fwd.add_zone(name.as_str(), &zone_settings).await
            .map(|_| ())
            .map_err(|e| std::io::Error::new(get_kind(&e), e))
    }
}

fn get_kind(zbus_error: &zbus::Error) -> std::io::ErrorKind {
    match zbus_error {
        zbus::Error::InterfaceNotFound => std::io::ErrorKind::NotFound,
        zbus::Error::Address(_) => std::io::ErrorKind::AddrNotAvailable,
        zbus::Error::InputOutput(_) => std::io::ErrorKind::Other,
        zbus::Error::InvalidField => std::io::ErrorKind::InvalidData,
        zbus::Error::ExcessData => std::io::ErrorKind::Other,
        zbus::Error::Variant(_) => std::io::ErrorKind::Other,
        zbus::Error::Names(_) => std::io::ErrorKind::Other,
        zbus::Error::IncorrectEndian => std::io::ErrorKind::Other,
        zbus::Error::Handshake(_) => std::io::ErrorKind::Other,
        zbus::Error::InvalidReply => std::io::ErrorKind::Other,
        zbus::Error::MethodError(_, _, _) => std::io::ErrorKind::InvalidInput,
        zbus::Error::MissingField => std::io::ErrorKind::Other,
        zbus::Error::InvalidGUID => std::io::ErrorKind::Other,
        zbus::Error::Unsupported => std::io::ErrorKind::Other,
        zbus::Error::FDO(_) => std::io::ErrorKind::Other,
        zbus::Error::NameTaken => std::io::ErrorKind::Other,
        zbus::Error::InvalidMatchRule => std::io::ErrorKind::Other,
        zbus::Error::Failure(_) => std::io::ErrorKind::Other,
        zbus::Error::MissingParameter(_) => std::io::ErrorKind::Other,
        zbus::Error::InvalidSerial => std::io::ErrorKind::Other,
        zbus::Error::InterfaceExists(_, _) => std::io::ErrorKind::AlreadyExists,
        _ => std::io::ErrorKind::Other,
    }
}
