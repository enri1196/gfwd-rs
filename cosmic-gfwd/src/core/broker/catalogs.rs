//! Network-interface, service, and ICMP-type catalog discovery.

use gfwd_bus::config_icmptype::ConfigIcmpTypeProxy;
use gfwd_bus::network_manager::{DeviceProxy, NetworkManagerProxy};

use crate::models::IcmpTypeInfo;

use super::{BrokerError, FwdBroker};

impl FwdBroker {
    /// Lists the permanent firewalld service catalog in name order.
    pub async fn get_services(&self) -> Result<Vec<String>, BrokerError> {
        let mut services = self.config().await?.get_service_names().await?;
        services.sort();
        Ok(services)
    }

    /// Lists configured ICMP types and descriptions in name order.
    pub async fn get_icmp_types(&self) -> Result<Vec<IcmpTypeInfo>, BrokerError> {
        let cfg = self.config().await?;
        let names = cfg.get_icmp_type_names().await?;
        let mut types = Vec::with_capacity(names.len());
        for name in names {
            let path = cfg.get_icmp_type_by_name(&name).await?;
            let proxy = ConfigIcmpTypeProxy::builder(&self.conn)
                .path(path)?
                .build()
                .await?;
            types.push(IcmpTypeInfo {
                description: proxy.get_description().await?,
                name,
            });
        }
        types.sort_by(|left, right| left.name.cmp(&right.name));
        Ok(types)
    }

    /// Discover usable network interfaces in stable name order.
    pub async fn get_interfaces(&self) -> Result<Vec<String>, BrokerError> {
        match self.get_interfaces_from_networkmanager().await {
            Ok(interfaces) if !interfaces.is_empty() => Ok(interfaces),
            Ok(_) | Err(_) => self.get_interfaces_from_sysfs().await,
        }
    }

    async fn get_interfaces_from_networkmanager(&self) -> Result<Vec<String>, BrokerError> {
        let proxy = NetworkManagerProxy::new(&self.conn).await?;
        let devices = proxy.get_devices().await?;
        let mut interfaces = Vec::with_capacity(devices.len());

        for path in devices {
            let device = DeviceProxy::builder(&self.conn)
                .path(path.as_str())?
                .build()
                .await?;
            let name = device.interface().await?;
            if should_include_interface(&name) {
                interfaces.push(name);
            }
        }

        interfaces.sort();
        interfaces.dedup();
        Ok(interfaces)
    }

    async fn get_interfaces_from_sysfs(&self) -> Result<Vec<String>, BrokerError> {
        use std::fs;

        let entries = match fs::read_dir("/sys/class/net") {
            Ok(entries) => entries,
            Err(error) => {
                eprintln!("failed to read /sys/class/net: {error}");
                return Ok(Vec::new());
            }
        };

        let mut interfaces = Vec::new();
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str()
                && should_include_interface(name)
            {
                interfaces.push(name.to_string());
            }
        }

        interfaces.sort();
        interfaces.dedup();
        Ok(interfaces)
    }
}

fn should_include_interface(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }

    !(name == "lo"
        || name.starts_with("docker")
        || name.starts_with("veth")
        || name.starts_with("br-"))
}
