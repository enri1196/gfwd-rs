use std::collections::HashMap;
use zbus::{
    Connection, Result as ZResult,
    zvariant::{OwnedObjectPath, OwnedValue},
};
use zbus_macros::proxy;

/// Type alias for permanent IPSet settings.
/// (version, name, description, type, options, entries)
pub type IPSetSettings = (
    String,
    String,
    String,
    String,
    HashMap<String, String>,
    Vec<String>,
);

/// Type alias for permanent ICMP type settings.
/// (version, name, description, destinations)
pub type IcmpTypeSettings = (String, String, String, Vec<String>);

/// Type alias for permanent service settings (deprecated).
/// (version, name, description, ports, modules, destinations, protocols, source-ports)
pub type ServiceSettings = (
    String,
    String,
    String,
    Vec<(String, String)>,
    Vec<String>,
    HashMap<String, String>,
    Vec<String>,
    Vec<(String, String)>,
);

/// Type alias for permanent zone settings.
pub type ZoneSettings = (
    String,                                // version
    String,                                // name
    String,                                // description
    bool,                                  // UNUSED
    String,                                // target
    Vec<String>,                           // services
    Vec<(String, String)>,                 // ports
    Vec<String>,                           // icmp-blocks
    bool,                                  // masquerade
    Vec<(String, String, String, String)>, // forward-ports
    Vec<String>,                           // interfaces
    Vec<String>,                           // sources
    Vec<String>,                           // rich rules
    Vec<String>,                           // protocols
    Vec<(String, String)>,                 // source-ports
);

#[proxy(
    interface = "org.fedoraproject.FirewallD1.config",
    gen_blocking = false,
    default_service = "org.fedoraproject.FirewallD1",
    default_path = "/org/fedoraproject/FirewallD1/config"
)]
/// Proxy for the `org.fedoraproject.FirewallD1.config` interface.
///
/// Allows to permanently add, remove and query zones, services and icmp types.
pub trait ConfigFirewalld1 {
    /// Add an IPSet to the permanent configuration.
    #[zbus(name = "addIPSet")]
    fn add_ipset(&self, ipset: &str, settings: &IPSetSettings) -> ZResult<OwnedObjectPath>;

    /// Add an ICMP type to the permanent configuration.
    #[zbus(name = "addIcmpType")]
    fn add_icmp_type(
        &self,
        icmptype: &str,
        settings: &IcmpTypeSettings,
    ) -> ZResult<OwnedObjectPath>;

    /// Deprecated. Use `add_service2` instead.
    #[deprecated(note = "Use `add_service2` instead.")]
    #[zbus(name = "addService")]
    fn add_service(&self, service: &str, settings: &ServiceSettings) -> ZResult<OwnedObjectPath>;

    /// Add a service to the permanent configuration.
    #[zbus(name = "addService2")]
    fn add_service2(
        &self,
        service: &str,
        settings: &HashMap<String, OwnedValue>,
    ) -> ZResult<OwnedObjectPath>;

    /// Add a zone to the permanent configuration.
    #[zbus(name = "addZone")]
    fn add_zone(&self, zone: &str, settings: &ZoneSettings) -> ZResult<OwnedObjectPath>;

    /// Get the object path of a helper by name.
    #[zbus(name = "getHelperByName")]
    fn get_helper_by_name(&self, helper: &str) -> ZResult<OwnedObjectPath>;

    /// Get the names of all permanent helpers.
    #[zbus(name = "getHelperNames")]
    fn get_helper_names(&self) -> ZResult<Vec<String>>;

    /// Get the object path of an IPSet by name.
    #[zbus(name = "getIPSetByName")]
    fn get_ipset_by_name(&self, ipset: &str) -> ZResult<OwnedObjectPath>;

    /// Get the names of all permanent IPSets.
    #[zbus(name = "getIPSetNames")]
    fn get_ipset_names(&self) -> ZResult<Vec<String>>;

    /// Get the object path of an ICMP type by name.
    #[zbus(name = "getIcmpTypeByName")]
    fn get_icmp_type_by_name(&self, icmptype: &str) -> ZResult<OwnedObjectPath>;

    /// Get the names of all permanent ICMP types.
    #[zbus(name = "getIcmpTypeNames")]
    fn get_icmp_type_names(&self) -> ZResult<Vec<String>>;

    /// Get the object path of a service by name.
    #[zbus(name = "getServiceByName")]
    fn get_service_by_name(&self, service: &str) -> ZResult<OwnedObjectPath>;

    /// Get the names of all permanent services.
    #[zbus(name = "getServiceNames")]
    fn get_service_names(&self) -> ZResult<Vec<String>>;

    /// Get the object path of a zone by name.
    #[zbus(name = "getZoneByName")]
    fn get_zone_by_name(&self, zone: &str) -> ZResult<OwnedObjectPath>;

    /// Get the names of all permanent zones.
    #[zbus(name = "getZoneNames")]
    fn get_zone_names(&self) -> ZResult<Vec<String>>;

    /// Get the name of the zone an interface is bound to.
    #[zbus(name = "getZoneOfInterface")]
    fn get_zone_of_interface(&self, iface: &str) -> ZResult<String>;

    /// Get the name of the zone a source is bound to.
    #[zbus(name = "getZoneOfSource")]
    fn get_zone_of_source(&self, source: &str) -> ZResult<String>;

    /// List object paths of all permanent helpers.
    #[zbus(name = "listHelpers")]
    fn list_helpers(&self) -> ZResult<Vec<OwnedObjectPath>>;

    /// List object paths of all permanent IPSets.
    #[zbus(name = "listIPSets")]
    fn list_ipsets(&self) -> ZResult<Vec<OwnedObjectPath>>;

    /// List object paths of all permanent ICMP types.
    #[zbus(name = "listIcmpTypes")]
    fn list_icmp_types(&self) -> ZResult<Vec<OwnedObjectPath>>;

    /// List object paths of all permanent services.
    #[zbus(name = "listServices")]
    fn list_services(&self) -> ZResult<Vec<OwnedObjectPath>>;

    /// List object paths of all permanent zones.
    #[zbus(name = "listZones")]
    fn list_zones(&self) -> ZResult<Vec<OwnedObjectPath>>;

    /// Signal: emitted when a helper has been added.
    #[zbus(signal, name = "HelperAdded")]
    fn helper_added(&self, helper: &str) -> ZResult<()>;

    /// Signal: emitted when an IPSet has been added.
    #[zbus(signal, name = "IPSetAdded")]
    fn ipset_added(&self, ipset: &str) -> ZResult<()>;

    /// Signal: emitted when an ICMP type has been added.
    #[zbus(signal, name = "IcmpTypeAdded")]
    fn icmp_type_added(&self, icmptype: &str) -> ZResult<()>;

    /// Signal: emitted when a service has been added.
    #[zbus(signal, name = "ServiceAdded")]
    fn service_added(&self, service: &str) -> ZResult<()>;

    /// Signal: emitted when a zone has been added.
    #[zbus(signal, name = "ZoneAdded")]
    fn zone_added(&self, zone: &str) -> ZResult<()>;

    /// Property: Allow zone drifting.
    #[zbus(property)]
    fn allow_zone_drifting(&self) -> ZResult<String>;
    #[zbus(property)]
    fn set_allow_zone_drifting(&self, value: &str) -> ZResult<()>;

    /// Property: Automatic kernel helper handling. Deprecated.
    #[zbus(property)]
    fn automatic_helpers(&self) -> ZResult<String>;
    #[zbus(property)]
    fn set_automatic_helpers(&self, value: &str) -> ZResult<()>;

    /// Property: Clean up firewall rules on exit.
    #[zbus(property)]
    fn cleanup_on_exit(&self) -> ZResult<String>;
    #[zbus(property)]
    fn set_cleanup_on_exit(&self, value: &str) -> ZResult<()>;

    /// Property: The default zone for connections or interfaces.
    #[zbus(property, name = "DefaultZone")]
    fn default_zone(&self) -> ZResult<String>;

    /// Property: The firewall backend.
    #[zbus(property)]
    fn firewall_backend(&self) -> ZResult<String>;
    #[zbus(property)]
    fn set_firewall_backend(&self, value: &str) -> ZResult<()>;

    /// Property: Flush all runtime rules on a reload.
    #[zbus(property)]
    fn flush_all_on_reload(&self) -> ZResult<String>;
    #[zbus(property)]
    fn set_flush_all_on_reload(&self, value: &str) -> ZResult<()>;

    /// Property: Enable IPv6 reverse path filter test.
    #[zbus(property, name = "IPv6_rpfilter")]
    fn ipv6_rpfilter(&self) -> ZResult<String>;
    #[zbus(property, name = "set_IPv6_rpfilter")]
    fn set_ipv6_rpfilter(&self, value: &str) -> ZResult<()>;

    /// Property: Use individual calls instead of restore calls.
    #[zbus(property, name = "IndividualCalls")]
    fn individual_calls(&self) -> ZResult<String>;

    /// Property: Enable or disable lockdown.
    #[zbus(property)]
    fn lockdown(&self) -> ZResult<String>;
    #[zbus(property)]
    fn set_lockdown(&self, value: &str) -> ZResult<()>;

    /// Property: Log denied packets.
    #[zbus(property)]
    fn log_denied(&self) -> ZResult<String>;
    #[zbus(property)]
    fn set_log_denied(&self, value: &str) -> ZResult<()>;

    /// Property: Minimal packet mark. Deprecated.
    #[zbus(property)]
    fn minimal_mark(&self) -> ZResult<i32>;
    #[zbus(property)]
    fn set_minimal_mark(&self, value: i32) -> ZResult<()>;

    /// Property: Filter RFC3964-violating 6to4 traffic.
    #[zbus(property)]
    fn rfc3964_ipv4(&self) -> ZResult<String>;
    #[zbus(property)]
    fn set_rfc3964_ipv4(&self, value: &str) -> ZResult<()>;
}

pub async fn new_config_firewalld1_proxy() -> ZResult<ConfigFirewalld1Proxy<'static>> {
    let conn = Connection::system().await?;
    ConfigFirewalld1Proxy::<'static>::new(&conn).await
}
