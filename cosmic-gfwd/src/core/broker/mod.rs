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
