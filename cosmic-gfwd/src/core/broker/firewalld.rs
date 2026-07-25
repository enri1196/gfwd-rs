//! Root firewalld and systemd status, control, and global operations.

use gfwd_bus::firewalld1::FirewallD1Proxy;
use gfwd_bus::systemd::{ManagerProxy, UnitProxy};

use super::{BrokerError, FirewalldStatus, FwdBroker};

impl FwdBroker {
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
    ///
    /// This is a global operation that applies all permanent configuration and
    /// discards runtime-only changes across every firewalld object.
    pub async fn apply_permanent_configuration(&self) -> Result<(), BrokerError> {
        FirewallD1Proxy::new(&self.conn).await?.reload().await?;
        Ok(())
    }

    /// Persist all current runtime firewalld configuration globally.
    ///
    /// This saves runtime state across every firewalld object, not only the
    /// currently selected zone.
    pub async fn persist_runtime_configuration(&self) -> Result<(), BrokerError> {
        FirewallD1Proxy::new(&self.conn)
            .await?
            .runtime_to_permanent()
            .await?;
        Ok(())
    }

    /// Return the configured default zone.
    pub async fn get_default_zone(&self) -> Result<String, BrokerError> {
        let cfg = self.config().await?;
        Ok(cfg.default_zone().await?)
    }

    /// Set the global runtime default zone.
    pub async fn set_default_zone(&self, zone_name: &str) -> Result<(), BrokerError> {
        let proxy = FirewallD1Proxy::new(&self.conn).await?;
        proxy.set_default_zone(zone_name).await?;
        Ok(())
    }
}
