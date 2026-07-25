//! Permanent IP-set listing, details, creation, deletion, and entry mutation.

use gfwd_bus::config_ipset::ConfigIPSetProxy;

use crate::models::IpSetDetails;

use super::{BrokerError, FwdBroker};

impl FwdBroker {
    /// List permanent IP sets in stable name order.
    pub async fn get_ipsets(&self) -> Result<Vec<String>, BrokerError> {
        let cfg = self.config().await?;
        let mut ipsets = cfg.get_ipset_names().await?;
        ipsets.sort();
        Ok(ipsets)
    }

    /// Load permanent details for one IP set.
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

    /// Permanently add an entry to an IP set.
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

    /// Permanently remove an entry from an IP set.
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

    /// Create a permanent IP set with its initial entries.
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
