use std::collections::HashMap;

use zbus::Result as ZResult;
use zbus_macros::proxy;
use zvariant::OwnedValue;

use crate::config_firewalld1::ConfigFirewalld1Proxy;

/// Type alias for permanent zone settings.
/// (version, name, description, UNUSED, target, services, ports, icmp-blocks,
/// masquerade, forward-ports, interfaces, sources, rich rules, protocols,
/// source-ports)
pub type ZoneSettings = (
    String,
    String,
    String,
    bool,
    String,
    Vec<String>,
    Vec<(String, String)>,
    Vec<String>,
    bool,
    Vec<(String, String, String, String)>,
    Vec<String>,
    Vec<String>,
    Vec<String>,
    Vec<String>,
    Vec<(String, String)>,
);

#[proxy(
    interface = "org.fedoraproject.FirewallD1.config.zone",
    gen_blocking = false,
    default_service = "org.fedoraproject.FirewallD1"
)]
/// Proxy for the `org.fedoraproject.FirewallD1.config.zone` interface.
///
/// Interface for permanent zone configuration.
pub trait ConfigZone {
    /// Permanently add a forward port to the zone.
    #[zbus(name = "addForwardPort")]
    fn add_forward_port(
        &self,
        port: &str,
        protocol: &str,
        to_port: &str,
        to_addr: &str,
    ) -> ZResult<()>;

    /// Permanently add an ICMP type to the list of blocked types in the zone.
    #[zbus(name = "addIcmpBlock")]
    fn add_icmp_block(&self, icmptype: &str) -> ZResult<()>;

    /// Permanently add ICMP block inversion to the zone.
    #[zbus(name = "addIcmpBlockInversion")]
    fn add_icmp_block_inversion(&self) -> ZResult<()>;

    /// Permanently add an interface to the list of interfaces bound to the zone.
    #[zbus(name = "addInterface")]
    fn add_interface(&self, interface: &str) -> ZResult<()>;

    /// Permanently enable masquerading in the zone.
    #[zbus(name = "addMasquerade")]
    fn add_masquerade(&self) -> ZResult<()>;

    /// Permanently add a port to the zone.
    #[zbus(name = "addPort")]
    fn add_port(&self, port: &str, protocol: &str) -> ZResult<()>;

    /// Permanently add a protocol to the zone.
    #[zbus(name = "addProtocol")]
    fn add_protocol(&self, protocol: &str) -> ZResult<()>;

    /// Permanently add a rich rule to the zone.
    #[zbus(name = "addRichRule")]
    fn add_rich_rule(&self, rule: &str) -> ZResult<()>;

    /// Permanently add a service to the zone.
    #[zbus(name = "addService")]
    fn add_service(&self, service: &str) -> ZResult<()>;

    /// Permanently add a source to the zone.
    #[zbus(name = "addSource")]
    fn add_source(&self, source: &str) -> ZResult<()>;

    /// Permanently add a source port to the zone.
    #[zbus(name = "addSourcePort")]
    fn add_source_port(&self, port: &str, protocol: &str) -> ZResult<()>;

    /// Get the description of the zone.
    #[zbus(name = "getDescription")]
    fn get_description(&self) -> ZResult<String>;

    /// Get the list of forward ports defined in the zone.
    #[zbus(name = "getForwardPorts")]
    fn get_forward_ports(&self) -> ZResult<Vec<(String, String, String, String)>>;

    /// Get the ICMP block inversion flag of the zone.
    #[zbus(name = "getIcmpBlockInversion")]
    fn get_icmp_block_inversion(&self) -> ZResult<bool>;

    /// Get the list of ICMP types blocked in the zone.
    #[zbus(name = "getIcmpBlocks")]
    fn get_icmp_blocks(&self) -> ZResult<Vec<String>>;

    /// Get the list of interfaces bound to the zone.
    #[zbus(name = "getInterfaces")]
    fn get_interfaces(&self) -> ZResult<Vec<String>>;

    /// Return whether masquerade is enabled in the zone.
    #[zbus(name = "getMasquerade")]
    fn get_masquerade(&self) -> ZResult<bool>;

    /// Get the list of ports defined in the zone.
    #[zbus(name = "getPorts")]
    fn get_ports(&self) -> ZResult<Vec<(String, String)>>;

    /// Get the array of protocols enabled in the zone.
    #[zbus(name = "getProtocols")]
    fn get_protocols(&self) -> ZResult<Vec<String>>;

    /// Get the list of rich language rules in the zone.
    #[zbus(name = "getRichRules")]
    fn get_rich_rules(&self) -> ZResult<Vec<String>>;

    /// Get the list of service names used in the zone.
    #[zbus(name = "getServices")]
    fn get_services(&self) -> ZResult<Vec<String>>;

    /// Get the permanent settings of the zone.
    #[zbus(name = "getSettings")]
    fn get_settings(&self) -> ZResult<ZoneSettings>;

    #[zbus(name = "getSettings2")]
    fn get_settings2(&self) -> ZResult<HashMap<String, OwnedValue>>;

    /// Get the short name of the zone.
    #[zbus(name = "getShort")]
    fn get_short(&self) -> ZResult<String>;

    /// Get the list of source ports defined in the zone.
    #[zbus(name = "getSourcePorts")]
    fn get_source_ports(&self) -> ZResult<Vec<(String, String)>>;

    /// Get the list of source addresses bound to the zone.
    #[zbus(name = "getSources")]
    fn get_sources(&self) -> ZResult<Vec<String>>;

    /// Get the target of the zone.
    #[zbus(name = "getTarget")]
    fn get_target(&self) -> ZResult<String>;

    /// Get the version of the zone.
    #[zbus(name = "getVersion")]
    fn get_version(&self) -> ZResult<String>;

    /// Load default settings for a built-in zone.
    #[zbus(name = "loadDefaults")]
    fn load_defaults(&self) -> ZResult<()>;

    /// Return whether a forward port is in the list of forward ports of the zone.
    #[zbus(name = "queryForwardPort")]
    fn query_forward_port(
        &self,
        port: &str,
        protocol: &str,
        to_port: &str,
        to_addr: &str,
    ) -> ZResult<bool>;

    /// Return whether an ICMP type is in the list of blocked types in the zone.
    #[zbus(name = "queryIcmpBlock")]
    fn query_icmp_block(&self, icmptype: &str) -> ZResult<bool>;

    /// Return whether ICMP block inversion is enabled in the zone.
    #[zbus(name = "queryIcmpBlockInversion")]
    fn query_icmp_block_inversion(&self) -> ZResult<bool>;

    /// Return whether an interface is in the list of interfaces bound to the zone.
    #[zbus(name = "queryInterface")]
    fn query_interface(&self, interface: &str) -> ZResult<bool>;

    /// Return whether masquerade is enabled in the zone.
    #[zbus(name = "queryMasquerade")]
    fn query_masquerade(&self) -> ZResult<bool>;

    /// Return whether a port is in the list of ports of the zone.
    #[zbus(name = "queryPort")]
    fn query_port(&self, port: &str, protocol: &str) -> ZResult<bool>;

    /// Return whether a protocol has been added to the zone.
    #[zbus(name = "queryProtocol")]
    fn query_protocol(&self, protocol: &str) -> ZResult<bool>;

    /// Return whether a rule is in the list of rich-language rules in the zone.
    #[zbus(name = "queryRichRule")]
    fn query_rich_rule(&self, rule: &str) -> ZResult<bool>;

    /// Return whether a service is in the list of services used in the zone.
    #[zbus(name = "queryService")]
    fn query_service(&self, service: &str) -> ZResult<bool>;

    /// Return whether a source is in the list of source addresses bound to the zone.
    #[zbus(name = "querySource")]
    fn query_source(&self, source: &str) -> ZResult<bool>;

    /// Return whether a source port is in the list of source ports of the zone.
    #[zbus(name = "querySourcePort")]
    fn query_source_port(&self, port: &str, protocol: &str) -> ZResult<bool>;

    /// Remove a non-built-in zone.
    #[zbus(name = "remove")]
    fn remove(&self) -> ZResult<()>;

    /// Permanently remove a forward port from the zone.
    #[zbus(name = "removeForwardPort")]
    fn remove_forward_port(
        &self,
        port: &str,
        protocol: &str,
        to_port: &str,
        to_addr: &str,
    ) -> ZResult<()>;

    /// Permanently remove an ICMP type from the list of blocked types in the zone.
    #[zbus(name = "removeIcmpBlock")]
    fn remove_icmp_block(&self, icmptype: &str) -> ZResult<()>;

    /// Permanently remove ICMP block inversion from the zone.
    #[zbus(name = "removeIcmpBlockInversion")]
    fn remove_icmp_block_inversion(&self) -> ZResult<()>;

    /// Permanently remove an interface from the list of interfaces bound to the zone.
    #[zbus(name = "removeInterface")]
    fn remove_interface(&self, interface: &str) -> ZResult<()>;

    /// Permanently disable masquerading in the zone.
    #[zbus(name = "removeMasquerade")]
    fn remove_masquerade(&self) -> ZResult<()>;

    /// Permanently remove a port from the zone.
    #[zbus(name = "removePort")]
    fn remove_port(&self, port: &str, protocol: &str) -> ZResult<()>;

    /// Permanently remove a protocol from the zone.
    #[zbus(name = "removeProtocol")]
    fn remove_protocol(&self, protocol: &str) -> ZResult<()>;

    /// Permanently remove a rich rule from the zone.
    #[zbus(name = "removeRichRule")]
    fn remove_rich_rule(&self, rule: &str) -> ZResult<()>;

    /// Permanently remove a service from the zone.
    #[zbus(name = "removeService")]
    fn remove_service(&self, service: &str) -> ZResult<()>;

    /// Permanently remove a source from the zone.
    #[zbus(name = "removeSource")]
    fn remove_source(&self, source: &str) -> ZResult<()>;

    /// Permanently remove a source port from the zone.
    #[zbus(name = "removeSourcePort")]
    fn remove_source_port(&self, port: &str, protocol: &str) -> ZResult<()>;

    /// Rename a non-built-in zone.
    #[zbus(name = "rename")]
    fn rename(&self, name: &str) -> ZResult<()>;

    /// Permanently set the description of the zone.
    #[zbus(name = "setDescription")]
    fn set_description(&self, description: &str) -> ZResult<()>;

    /// Permanently set the forward ports of the zone.
    #[zbus(name = "setForwardPorts")]
    fn set_forward_ports(&self, ports: &[(String, String, String, String)]) -> ZResult<()>;

    /// Permanently set the ICMP block inversion flag of the zone.
    #[zbus(name = "setIcmpBlockInversion")]
    fn set_icmp_block_inversion(&self, flag: bool) -> ZResult<()>;

    /// Permanently set the list of ICMP types blocked in the zone.
    #[zbus(name = "setIcmpBlocks")]
    fn set_icmp_blocks(&self, icmptypes: &[String]) -> ZResult<()>;

    /// Permanently set the list of interfaces bound to the zone.
    #[zbus(name = "setInterfaces")]
    fn set_interfaces(&self, interfaces: &[String]) -> ZResult<()>;

    /// Permanently set masquerading in the zone.
    #[zbus(name = "setMasquerade")]
    fn set_masquerade(&self, masquerade: bool) -> ZResult<()>;

    /// Permanently set the ports of the zone.
    #[zbus(name = "setPorts")]
    fn set_ports(&self, ports: &[(String, String)]) -> ZResult<()>;

    /// Permanently set the list of protocols used in the zone.
    #[zbus(name = "setProtocols")]
    fn set_protocols(&self, protocols: &[String]) -> ZResult<()>;

    /// Permanently set the list of rich-language rules.
    #[zbus(name = "setRichRules")]
    fn set_rich_rules(&self, rules: &[String]) -> ZResult<()>;

    /// Permanently set the list of services used in the zone.
    #[zbus(name = "setServices")]
    fn set_services(&self, services: &[String]) -> ZResult<()>;

    /// Permanently set the short name of the zone.
    #[zbus(name = "setShort")]
    fn set_short(&self, short: &str) -> ZResult<()>;

    /// Permanently set the source ports of the zone.
    #[zbus(name = "setSourcePorts")]
    fn set_source_ports(&self, ports: &[(String, String)]) -> ZResult<()>;

    /// Permanently set the list of source addresses bound to the zone.
    #[zbus(name = "setSources")]
    fn set_sources(&self, sources: &[String]) -> ZResult<()>;

    /// Permanently set the target of the zone.
    #[zbus(name = "setTarget")]
    fn set_target(&self, target: &str) -> ZResult<()>;

    /// Permanently set the version of the zone.
    #[zbus(name = "setVersion")]
    fn set_version(&self, version: &str) -> ZResult<()>;

    /// Update the settings of the zone.
    #[zbus(name = "update")]
    fn update(&self, settings: &ZoneSettings) -> ZResult<()>;

    /// Signal: emitted when the zone has been removed.
    #[zbus(signal, name = "Removed")]
    fn removed(&self, name: &str) -> ZResult<()>;

    /// Signal: emitted when the zone has been renamed.
    #[zbus(signal, name = "Renamed")]
    fn renamed(&self, name: &str) -> ZResult<()>;

    /// Signal: emitted when the zone has been updated.
    #[zbus(signal, name = "Updated")]
    fn updated(&self, name: &str) -> ZResult<()>;

    /// Property: True if the zone is built-in.
    #[zbus(property, name = "Builtin")]
    fn builtin(&self) -> ZResult<bool>;

    /// Property: True if a built-in zone has default settings.
    #[zbus(property, name = "Default")]
    fn default(&self) -> ZResult<bool>;

    /// Property: The name of the configuration file.
    #[zbus(property, name = "Filename")]
    fn filename(&self) -> ZResult<String>;

    /// Property: The name of the zone.
    #[zbus(property, name = "Name")]
    fn name(&self) -> ZResult<String>;

    /// Property: The path to the configuration directory.
    #[zbus(property, name = "Path")]
    fn path(&self) -> ZResult<String>;
}

/// Creates a new proxy for a specific zone's permanent configuration.
///
/// # Arguments
///
/// * `conn` - An active zbus connection.
/// * `zone_name` - The name of the zone to configure (e.g., "public").
#[deprecated(note = "Use ConfigFirewalld1Proxy::get_zone_by_name + ConfigZoneProxy::builder on shared Connection")]
pub async fn new_config_zone_proxy(
    _config_proxy: &ConfigFirewalld1Proxy<'_>,
    _zone_name: &str,
) -> ZResult<ConfigZoneProxy<'static>> {
    // Deprecated in favor of resolving via get_zone_by_name and building a proxy on a shared connection.
    unreachable!("Use ConfigFirewalld1Proxy::get_zone_by_name + ConfigZoneProxy::builder on shared Connection");
}
