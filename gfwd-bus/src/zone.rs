use std::collections::HashMap;
use zbus::{Connection, Result as ZResult};
use zbus_macros::proxy;

#[proxy(
    interface = "org.fedoraproject.FirewallD1.zone",
    gen_blocking = false,
    default_service = "org.fedoraproject.FirewallD1",
    default_path = "/org/fedoraproject/FirewallD1"
)]
/// Proxy for the `org.fedoraproject.FirewallD1.zone` interface.
///
/// Operations in this interface allows to get, add, remove and query runtime
/// zone's settings. For permanent settings see
/// `org.fedoraproject.FirewallD1.config.zone` interface.
pub trait Zone {
    /// Add an IPv4 forward port to a zone.
    #[zbus(name = "addForwardPort")]
    fn add_forward_port(
        &self,
        zone: &str,
        port: &str,
        protocol: &str,
        to_port: &str,
        to_addr: &str,
        timeout: i32,
    ) -> ZResult<String>;

    /// Add an ICMP block to a zone.
    #[zbus(name = "addIcmpBlock")]
    fn add_icmp_block(&self, zone: &str, icmp: &str, timeout: i32) -> ZResult<String>;

    /// Add ICMP block inversion to a zone.
    #[zbus(name = "addIcmpBlockInversion")]
    fn add_icmp_block_inversion(&self, zone: &str) -> ZResult<String>;

    /// Bind an interface to a zone.
    #[zbus(name = "addInterface")]
    fn add_interface(&self, zone: &str, interface: &str) -> ZResult<String>;

    /// Enable masquerade in a zone.
    #[zbus(name = "addMasquerade")]
    fn add_masquerade(&self, zone: &str, timeout: i32) -> ZResult<String>;

    /// Add a port to a zone.
    #[zbus(name = "addPort")]
    fn add_port(&self, zone: &str, port: &str, protocol: &str, timeout: i32) -> ZResult<String>;

    /// Add a protocol to a zone.
    #[zbus(name = "addProtocol")]
    fn add_protocol(&self, zone: &str, protocol: &str, timeout: i32) -> ZResult<String>;

    /// Add a rich language rule to a zone.
    #[zbus(name = "addRichRule")]
    fn add_rich_rule(&self, zone: &str, rule: &str, timeout: i32) -> ZResult<String>;

    /// Add a service to a zone.
    #[zbus(name = "addService")]
    fn add_service(&self, zone: &str, service: &str, timeout: i32) -> ZResult<String>;

    /// Bind a source to a zone.
    #[zbus(name = "addSource")]
    fn add_source(&self, zone: &str, source: &str) -> ZResult<String>;

    /// Add a source port to a zone.
    #[zbus(name = "addSourcePort")]
    fn add_source_port(
        &self,
        zone: &str,
        port: &str,
        protocol: &str,
        timeout: i32,
    ) -> ZResult<String>;

    /// Deprecated. Use `change_zone_of_interface` instead.
    #[deprecated(note = "Use `change_zone_of_interface` instead.")]
    #[zbus(name = "changeZone")]
    fn change_zone(&self, zone: &str, interface: &str) -> ZResult<String>;

    /// Change the zone an interface is bound to.
    #[zbus(name = "changeZoneOfInterface")]
    fn change_zone_of_interface(&self, zone: &str, interface: &str) -> ZResult<String>;

    /// Change the zone a source is bound to.
    #[zbus(name = "changeZoneOfSource")]
    fn change_zone_of_source(&self, zone: &str, source: &str) -> ZResult<String>;

    /// Get all currently active zones.
    #[zbus(name = "getActiveZones")]
    fn get_active_zones(&self) -> ZResult<HashMap<String, HashMap<String, Vec<String>>>>;

    /// Get the IPv4 forward ports for a zone.
    #[zbus(name = "getForwardPorts")]
    fn get_forward_ports(&self, zone: &str) -> ZResult<Vec<(String, String, String, String)>>;

    /// Get the ICMP blocks for a zone.
    #[zbus(name = "getIcmpBlocks")]
    fn get_icmp_blocks(&self, zone: &str) -> ZResult<Vec<String>>;

    /// Get whether ICMP block inversion is enabled for a zone.
    #[zbus(name = "getIcmpBlockInversion")]
    fn get_icmp_block_inversion(&self, zone: &str) -> ZResult<bool>;

    /// Get the interfaces bound to a zone.
    #[zbus(name = "getInterfaces")]
    fn get_interfaces(&self, zone: &str) -> ZResult<Vec<String>>;

    /// Get the ports for a zone.
    #[zbus(name = "getPorts")]
    fn get_ports(&self, zone: &str) -> ZResult<Vec<(String, String)>>;

    /// Get the protocols for a zone.
    #[zbus(name = "getProtocols")]
    fn get_protocols(&self, zone: &str) -> ZResult<Vec<String>>;

    /// Get the rich rules for a zone.
    #[zbus(name = "getRichRules")]
    fn get_rich_rules(&self, zone: &str) -> ZResult<Vec<String>>;

    /// Get the services for a zone.
    #[zbus(name = "getServices")]
    fn get_services(&self, zone: &str) -> ZResult<Vec<String>>;

    /// Get the source ports for a zone.
    #[zbus(name = "getSourcePorts")]
    fn get_source_ports(&self, zone: &str) -> ZResult<Vec<(String, String)>>;

    /// Get the sources for a zone.
    #[zbus(name = "getSources")]
    fn get_sources(&self, zone: &str) -> ZResult<Vec<String>>;

    /// Get the zone an interface is bound to.
    #[zbus(name = "getZoneOfInterface")]
    fn get_zone_of_interface(&self, interface: &str) -> ZResult<String>;

    /// Get the zone a source is bound to.
    #[zbus(name = "getZoneOfSource")]
    fn get_zone_of_source(&self, source: &str) -> ZResult<String>;

    /// Get all predefined zones.
    #[zbus(name = "getZones")]
    fn get_zones(&self) -> ZResult<Vec<String>>;

    /// Deprecated.
    #[deprecated]
    #[zbus(name = "isImmutable")]
    fn is_immutable(&self, zone: &str) -> ZResult<bool>;

    /// Query whether an IPv4 forward port has been added to a zone.
    #[zbus(name = "queryForwardPort")]
    fn query_forward_port(
        &self,
        zone: &str,
        port: &str,
        protocol: &str,
        to_port: &str,
        to_addr: &str,
    ) -> ZResult<bool>;

    /// Query whether an ICMP block has been added to a zone.
    #[zbus(name = "queryIcmpBlock")]
    fn query_icmp_block(&self, zone: &str, icmp: &str) -> ZResult<bool>;

    /// Query whether ICMP block inversion has been added to a zone.
    #[zbus(name = "queryIcmpBlockInversion")]
    fn query_icmp_block_inversion(&self, zone: &str) -> ZResult<bool>;

    /// Query whether an interface is bound to a zone.
    #[zbus(name = "queryInterface")]
    fn query_interface(&self, zone: &str, interface: &str) -> ZResult<bool>;

    /// Query whether masquerading is enabled in a zone.
    #[zbus(name = "queryMasquerade")]
    fn query_masquerade(&self, zone: &str) -> ZResult<bool>;

    /// Query whether a port has been added to a zone.
    #[zbus(name = "queryPort")]
    fn query_port(&self, zone: &str, port: &str, protocol: &str) -> ZResult<bool>;

    /// Query whether a protocol has been added to a zone.
    #[zbus(name = "queryProtocol")]
    fn query_protocol(&self, zone: &str, protocol: &str) -> ZResult<bool>;

    /// Query whether a rich rule has been added to a zone.
    #[zbus(name = "queryRichRule")]
    fn query_rich_rule(&self, zone: &str, rule: &str) -> ZResult<bool>;

    /// Query whether a service has been added to a zone.
    #[zbus(name = "queryService")]
    fn query_service(&self, zone: &str, service: &str) -> ZResult<bool>;

    /// Query whether a source is bound to a zone.
    #[zbus(name = "querySource")]
    fn query_source(&self, zone: &str, source: &str) -> ZResult<bool>;

    /// Query whether a source port has been added to a zone.
    #[zbus(name = "querySourcePort")]
    fn query_source_port(&self, zone: &str, port: &str, protocol: &str) -> ZResult<bool>;

    /// Remove an IPv4 forward port from a zone.
    #[zbus(name = "removeForwardPort")]
    fn remove_forward_port(
        &self,
        zone: &str,
        port: &str,
        protocol: &str,
        to_port: &str,
        to_addr: &str,
    ) -> ZResult<String>;

    /// Remove an ICMP block from a zone.
    #[zbus(name = "removeIcmpBlock")]
    fn remove_icmp_block(&self, zone: &str, icmp: &str) -> ZResult<String>;

    /// Remove ICMP block inversion from a zone.
    #[zbus(name = "removeIcmpBlockInversion")]
    fn remove_icmp_block_inversion(&self, zone: &str) -> ZResult<String>;

    /// Remove an interface binding from a zone.
    #[zbus(name = "removeInterface")]
    fn remove_interface(&self, zone: &str, interface: &str) -> ZResult<String>;

    /// Disable masquerade in a zone.
    #[zbus(name = "removeMasquerade")]
    fn remove_masquerade(&self, zone: &str) -> ZResult<String>;

    /// Remove a port from a zone.
    #[zbus(name = "removePort")]
    fn remove_port(&self, zone: &str, port: &str, protocol: &str) -> ZResult<String>;

    /// Remove a protocol from a zone.
    #[zbus(name = "removeProtocol")]
    fn remove_protocol(&self, zone: &str, protocol: &str) -> ZResult<String>;

    /// Remove a rich rule from a zone.
    #[zbus(name = "removeRichRule")]
    fn remove_rich_rule(&self, zone: &str, rule: &str) -> ZResult<String>;

    /// Remove a service from a zone.
    #[zbus(name = "removeService")]
    fn remove_service(&self, zone: &str, service: &str) -> ZResult<String>;

    /// Remove a source binding from a zone.
    #[zbus(name = "removeSource")]
    fn remove_source(&self, zone: &str, source: &str) -> ZResult<String>;

    /// Remove a source port from a zone.
    #[zbus(name = "removeSourcePort")]
    fn remove_source_port(&self, zone: &str, port: &str, protocol: &str) -> ZResult<String>;

    /// Signal: emitted when a forward port has been added.
    #[zbus(signal, name = "ForwardPortAdded")]
    fn forward_port_added(
        &self,
        zone: &str,
        port: &str,
        protocol: &str,
        to_port: &str,
        to_addr: &str,
        timeout: i32,
    ) -> ZResult<()>;

    /// Signal: emitted when a forward port has been removed.
    #[zbus(signal, name = "ForwardPortRemoved")]
    fn forward_port_removed(
        &self,
        zone: &str,
        port: &str,
        protocol: &str,
        to_port: &str,
        to_addr: &str,
    ) -> ZResult<()>;

    /// Signal: emitted when an ICMP block has been added.
    #[zbus(signal, name = "IcmpBlockAdded")]
    fn icmp_block_added(&self, zone: &str, icmp: &str, timeout: i32) -> ZResult<()>;

    /// Signal: emitted when ICMP block inversion has been added.
    #[zbus(signal, name = "IcmpBlockInversionAdded")]
    fn icmp_block_inversion_added(&self, zone: &str) -> ZResult<()>;

    /// Signal: emitted when ICMP block inversion has been removed.
    #[zbus(signal, name = "IcmpBlockInversionRemoved")]
    fn icmp_block_inversion_removed(&self, zone: &str) -> ZResult<()>;

    /// Signal: emitted when an ICMP block has been removed.
    #[zbus(signal, name = "IcmpBlockRemoved")]
    fn icmp_block_removed(&self, zone: &str, icmp: &str) -> ZResult<()>;

    /// Signal: emitted when an interface has been added to a zone.
    #[zbus(signal, name = "InterfaceAdded")]
    fn interface_added(&self, zone: &str, interface: &str) -> ZResult<()>;

    /// Signal: emitted when an interface has been removed from a zone.
    #[zbus(signal, name = "InterfaceRemoved")]
    fn interface_removed(&self, zone: &str, interface: &str) -> ZResult<()>;

    /// Signal: emitted when masquerade has been enabled for a zone.
    #[zbus(signal, name = "MasqueradeAdded")]
    fn masquerade_added(&self, zone: &str, timeout: i32) -> ZResult<()>;

    /// Signal: emitted when masquerade has been disabled for a zone.
    #[zbus(signal, name = "MasqueradeRemoved")]
    fn masquerade_removed(&self, zone: &str) -> ZResult<()>;

    /// Signal: emitted when a port has been added to a zone.
    #[zbus(signal, name = "PortAdded")]
    fn port_added(&self, zone: &str, port: &str, protocol: &str, timeout: i32) -> ZResult<()>;

    /// Signal: emitted when a port has been removed from a zone.
    #[zbus(signal, name = "PortRemoved")]
    fn port_removed(&self, zone: &str, port: &str, protocol: &str) -> ZResult<()>;

    /// Signal: emitted when a protocol has been added to a zone.
    #[zbus(signal, name = "ProtocolAdded")]
    fn protocol_added(&self, zone: &str, protocol: &str, timeout: i32) -> ZResult<()>;

    /// Signal: emitted when a protocol has been removed from a zone.
    #[zbus(signal, name = "ProtocolRemoved")]
    fn protocol_removed(&self, zone: &str, protocol: &str) -> ZResult<()>;

    /// Signal: emitted when a rich rule has been added to a zone.
    #[zbus(signal, name = "RichRuleAdded")]
    fn rich_rule_added(&self, zone: &str, rule: &str, timeout: i32) -> ZResult<()>;

    /// Signal: emitted when a rich rule has been removed from a zone.
    #[zbus(signal, name = "RichRuleRemoved")]
    fn rich_rule_removed(&self, zone: &str, rule: &str) -> ZResult<()>;

    /// Signal: emitted when a service has been added to a zone.
    #[zbus(signal, name = "ServiceAdded")]
    fn service_added(&self, zone: &str, service: &str, timeout: i32) -> ZResult<()>;

    /// Signal: emitted when a service has been removed from a zone.
    #[zbus(signal, name = "ServiceRemoved")]
    fn service_removed(&self, zone: &str, service: &str) -> ZResult<()>;

    /// Signal: emitted when a source has been added to a zone.
    #[zbus(signal, name = "SourceAdded")]
    fn source_added(&self, zone: &str, source: &str) -> ZResult<()>;

    /// Signal: emitted when a source port has been added to a zone.
    #[zbus(signal, name = "SourcePortAdded")]
    fn source_port_added(
        &self,
        zone: &str,
        port: &str,
        protocol: &str,
        timeout: i32,
    ) -> ZResult<()>;

    /// Signal: emitted when a source port has been removed from a zone.
    #[zbus(signal, name = "SourcePortRemoved")]
    fn source_port_removed(&self, zone: &str, port: &str, protocol: &str) -> ZResult<()>;

    /// Signal: emitted when a source has been removed from a zone.
    #[zbus(signal, name = "SourceRemoved")]
    fn source_removed(&self, zone: &str, source: &str) -> ZResult<()>;

    /// Deprecated.
    #[deprecated]
    #[zbus(signal, name = "ZoneChanged")]
    fn zone_changed(&self, zone: &str, interface: &str) -> ZResult<()>;

    /// Signal: emitted when an interface's zone has changed.
    #[zbus(signal, name = "ZoneOfInterfaceChanged")]
    fn zone_of_interface_changed(&self, zone: &str, interface: &str) -> ZResult<()>;

    /// Signal: emitted when a source's zone has changed.
    #[zbus(signal, name = "ZoneOfSourceChanged")]
    fn zone_of_source_changed(&self, zone: &str, source: &str) -> ZResult<()>;
}

pub async fn new_zone_proxy() -> ZResult<ZoneProxy<'static>> {
    let conn = Connection::system().await?;
    ZoneProxy::<'static>::new(&conn).await
}
