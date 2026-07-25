//! Broker-owned D-Bus boundary for firewalld and related system services.
//!
//! All system-bus connections, proxy construction, and signal streams live in
//! this module tree. Application and UI code consume only the typed broker
//! methods exposed by [`FwdBroker`].

mod catalogs;
mod events;
mod firewalld;
mod ipset;
mod zone;

use gfwd_bus::config_firewalld1::ConfigFirewalld1Proxy;
use tokio::sync::OnceCell;
use zbus::Connection;

use super::reconciliation::ZoneSettingsParseError;

/// Error returned by broker-owned system-bus operations.
#[derive(Clone, Debug)]
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
    pub(super) fn new(message: impl Into<String>) -> Self {
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

/// Shared owner of all firewalld, systemd, and NetworkManager D-Bus proxies.
#[derive(Debug)]
pub struct FwdBroker {
    pub(super) conn: Connection,
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

    pub(super) async fn config(&self) -> Result<ConfigFirewalld1Proxy<'_>, BrokerError> {
        Ok(ConfigFirewalld1Proxy::new(&self.conn).await?)
    }
}

#[cfg(test)]
mod tests {
    use crate::models::ZoneTarget;

    use super::FwdBroker;

    async fn assert_public_call_surface(broker: &FwdBroker, target: &ZoneTarget) {
        let _ = broker.permanent_zone_snapshot("public").await;
        let _ = broker.runtime_zone_snapshot("public").await;
        let _ = broker.reconcile_zone("public").await;
        let _ = broker.configuration_events(Some("public".into()));
        let _ = broker.set_masquerade("public", true).await;
        let _ = broker.set_icmp_block_inversion("public", true).await;
        let _ = broker.firewalld_status().await;
        let _ = broker.start_firewalld().await;
        let _ = broker.stop_firewalld().await;
        let _ = broker.apply_permanent_configuration().await;
        let _ = broker.persist_runtime_configuration().await;
        let _ = broker.get_zones().await;
        let _ = broker.get_services().await;
        let _ = broker.get_icmp_types().await;
        let _ = broker.add_service("public", "ssh").await;
        let _ = broker.get_default_zone().await;
        let _ = broker.set_default_zone("public").await;
        let _ = broker.get_active_zones().await;
        let _ = broker.get_interfaces().await;
        let _ = broker.add_zone("test", "test zone", target).await;
        let _ = broker.add_port("public", "443", "tcp").await;
        let _ = broker.add_source_port("public", "443", "tcp").await;
        let _ = broker
            .add_forward_port("public", "443", "tcp", "8443", "")
            .await;
        let _ = broker.add_interface("public", "eth0").await;
        let _ = broker.add_source("public", "192.0.2.0/24").await;
        let _ = broker.add_icmp_block("public", "echo-request").await;
        let _ = broker.add_rich_rule("public", "rule accept").await;
        let _ = broker.remove_zone("test").await;
        let _ = broker.remove_service("public", "ssh").await;
        let _ = broker.remove_interface("public", "eth0").await;
        let _ = broker.remove_source("public", "192.0.2.0/24").await;
        let _ = broker.remove_port("public", "443", "tcp").await;
        let _ = broker
            .remove_forward_port("public", "443", "tcp", "8443", "")
            .await;
        let _ = broker.remove_source_port("public", "443", "tcp").await;
        let _ = broker.remove_icmp_block("public", "echo-request").await;
        let _ = broker.remove_rich_rule("public", "rule accept").await;
        let _ = broker.get_zone_details("public").await;
        let _ = broker.get_ipsets().await;
        let _ = broker.get_ipset_details("test").await;
        let _ = broker.add_ipset_entry("test", "192.0.2.1").await;
        let _ = broker.remove_ipset_entry("test", "192.0.2.1").await;
        let _ = broker.remove_ipset("test").await;
        let _ = broker
            .create_ipset("test", "hash:ip", vec!["192.0.2.1".into()])
            .await;
    }

    #[test]
    fn broker_public_call_surface_is_preserved() {
        let _ = FwdBroker::get;
        let _ = assert_public_call_surface;
    }
}
