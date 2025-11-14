use std::collections::HashMap;
use zbus::Result as ZResult;
use zbus_macros::proxy;
use zvariant::OwnedValue;

#[proxy(
    interface = "org.fedoraproject.FirewallD1.config.service",
    gen_blocking = false,
    default_service = "org.fedoraproject.FirewallD1"
)]
/// Proxy for the `org.fedoraproject.FirewallD1.config.service` interface.
///
/// Interface for permanent service configuration.
pub trait ConfigService {
    /// Permanently add a port to the service.
    #[zbus(name = "addPort")]
    fn add_port(&self, port: &str, protocol: &str) -> ZResult<()>;

    /// Permanently add a protocol to the service.
    #[zbus(name = "addProtocol")]
    fn add_protocol(&self, protocol: &str) -> ZResult<()>;

    /// Permanently add a source port to the service.
    #[zbus(name = "addSourcePort")]
    fn add_source_port(&self, port: &str, protocol: &str) -> ZResult<()>;

    /// Get the description of the service.
    #[zbus(name = "getDescription")]
    fn get_description(&self) -> ZResult<String>;

    /// Get the destination for an IP family.
    #[zbus(name = "getDestination")]
    fn get_destination(&self, family: &str) -> ZResult<String>;

    /// Get the list of destinations for the service.
    #[zbus(name = "getDestinations")]
    fn get_destinations(&self) -> ZResult<HashMap<String, String>>;

    /// Get the list of ports defined in the service.
    #[zbus(name = "getPorts")]
    fn get_ports(&self) -> ZResult<Vec<(String, String)>>;

    /// Get the array of protocols defined in the service.
    #[zbus(name = "getProtocols")]
    fn get_protocols(&self) -> ZResult<Vec<String>>;

    /// Get the settings of the service.
    #[zbus(name = "getSettings2")]
    fn get_settings2(&self) -> ZResult<HashMap<String, OwnedValue>>;

    /// Get the short name of the service.
    #[zbus(name = "getShort")]
    fn get_short(&self) -> ZResult<String>;

    /// Get the list of source ports defined in the service.
    #[zbus(name = "getSourcePorts")]
    fn get_source_ports(&self) -> ZResult<Vec<(String, String)>>;

    /// Get the version of the service.
    #[zbus(name = "getVersion")]
    fn get_version(&self) -> ZResult<String>;

    /// Load default settings for a built-in service.
    #[zbus(name = "loadDefaults")]
    fn load_defaults(&self) -> ZResult<()>;

    /// Return whether a destination is in the dictionary of destinations.
    #[zbus(name = "queryDestination")]
    fn query_destination(&self, family: &str, address: &str) -> ZResult<bool>;

    /// Return whether a port is in the list of ports in the service.
    #[zbus(name = "queryPort")]
    fn query_port(&self, port: &str, protocol: &str) -> ZResult<bool>;

    /// Return whether a protocol is in the list of protocols in the service.
    #[zbus(name = "queryProtocol")]
    fn query_protocol(&self, protocol: &str) -> ZResult<bool>;

    /// Return whether a source port is in the list of source ports in the service.
    #[zbus(name = "querySourcePort")]
    fn query_source_port(&self, port: &str, protocol: &str) -> ZResult<bool>;

    /// Remove a non-built-in service.
    #[zbus(name = "remove")]
    fn remove(&self) -> ZResult<()>;

    /// Permanently remove a destination from the service.
    #[zbus(name = "removeDestination")]
    fn remove_destination(&self, family: &str) -> ZResult<()>;

    /// Permanently remove a port from the service.
    #[zbus(name = "removePort")]
    fn remove_port(&self, port: &str, protocol: &str) -> ZResult<()>;

    /// Permanently remove a protocol from the service.
    #[zbus(name = "removeProtocol")]
    fn remove_protocol(&self, protocol: &str) -> ZResult<()>;

    /// Permanently remove a source port from the service.
    #[zbus(name = "removeSourcePort")]
    fn remove_source_port(&self, port: &str, protocol: &str) -> ZResult<()>;

    /// Rename a non-built-in service.
    #[zbus(name = "rename")]
    fn rename(&self, name: &str) -> ZResult<()>;

    /// Permanently set the description of the service.
    #[zbus(name = "setDescription")]
    fn set_description(&self, description: &str) -> ZResult<()>;

    /// Permanently set a destination address for the service.
    #[zbus(name = "setDestination")]
    fn set_destination(&self, family: &str, address: &str) -> ZResult<()>;

    /// Permanently set the destinations of the service.
    #[zbus(name = "setDestinations")]
    fn set_destinations(&self, destinations: &HashMap<String, String>) -> ZResult<()>;

    /// Permanently set the ports of the service.
    #[zbus(name = "setPorts")]
    fn set_ports(&self, ports: Vec<(String, String)>) -> ZResult<()>;

    /// Permanently set the protocols of the service.
    #[zbus(name = "setProtocols")]
    fn set_protocols(&self, protocols: Vec<String>) -> ZResult<()>;

    /// Permanently set the short name of the service.
    #[zbus(name = "setShort")]
    fn set_short(&self, short: &str) -> ZResult<()>;

    /// Permanently set the source ports of the service.
    #[zbus(name = "setSourcePorts")]
    fn set_source_ports(&self, ports: Vec<(String, String)>) -> ZResult<()>;

    /// Permanently set the version of the service.
    #[zbus(name = "setVersion")]
    fn set_version(&self, version: &str) -> ZResult<()>;

    /// Update the settings of the service.
    #[zbus(name = "update2")]
    fn update2(&self, settings: &HashMap<String, OwnedValue>) -> ZResult<()>;

    /// Signal: emitted when the service has been removed.
    #[zbus(signal, name = "Removed")]
    fn removed(&self, name: &str) -> ZResult<()>;

    /// Signal: emitted when the service has been renamed.
    #[zbus(signal, name = "Renamed")]
    fn renamed(&self, name: &str) -> ZResult<()>;

    /// Signal: emitted when the service has been updated.
    #[zbus(signal, name = "Updated")]
    fn updated(&self, name: &str) -> ZResult<()>;

    /// Property: True if the service is built-in.
    #[zbus(property)]
    fn builtin(&self) -> ZResult<bool>;

    /// Property: True if a built-in service has default settings.
    #[zbus(property)]
    fn default(&self) -> ZResult<bool>;

    /// Property: The name of the configuration file.
    #[zbus(property)]
    fn filename(&self) -> ZResult<String>;

    /// Property: The name of the service.
    #[zbus(property)]
    fn name(&self) -> ZResult<String>;

    /// Property: The path to the configuration directory.
    #[zbus(property)]
    fn path(&self) -> ZResult<String>;
}
