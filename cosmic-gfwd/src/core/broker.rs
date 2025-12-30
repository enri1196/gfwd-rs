use gfwd_bus::config_firewalld1::ConfigFirewalld1Proxy;
use gfwd_bus::config_zone::ConfigZoneProxy;
use tokio::sync::OnceCell;
use zbus::Connection;

use crate::models::{ZoneDetails, ZoneTarget};

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
