use zbus::Result as ZResult;
use zbus_macros::proxy;

/// Type alias for permanent helper settings.
/// (version, name, description, family, module, ports)
pub type HelperSettings = (
    String,
    String,
    String,
    String,
    String,
    Vec<(String, String)>,
);

#[proxy(
    interface = "org.fedoraproject.FirewallD1.config.helper",
    gen_blocking = false,
    default_service = "org.fedoraproject.FirewallD1",
    default_path = "/org/fedoraproject/FirewallD1/config/helper"
)]
/// Proxy for the `org.fedoraproject.FirewallD1.config.helper` interface.
///
/// Interface for permanent helper configuration.
pub trait ConfigHelper {
    /// Permanently add a port to the helper.
    #[zbus(name = "addPort")]
    fn add_port(&self, port: &str, protocol: &str) -> ZResult<()>;

    /// Get the description of the helper.
    #[zbus(name = "getDescription")]
    fn get_description(&self) -> ZResult<String>;

    /// Get the family of the helper.
    #[zbus(name = "getFamily")]
    fn get_family(&self) -> ZResult<String>;

    /// Get the module used in the helper.
    #[zbus(name = "getModule")]
    fn get_module(&self) -> ZResult<String>;

    /// Get the list of ports defined in the helper.
    #[zbus(name = "getPorts")]
    fn get_ports(&self) -> ZResult<Vec<(String, String)>>;

    /// Get the permanent settings of the helper.
    #[zbus(name = "getSettings")]
    fn get_settings(&self) -> ZResult<HelperSettings>;

    /// Get the short name of the helper.
    #[zbus(name = "getShort")]
    fn get_short(&self) -> ZResult<String>;

    /// Get the version of the helper.
    #[zbus(name = "getVersion")]
    fn get_version(&self) -> ZResult<String>;

    /// Load default settings for a built-in helper.
    #[zbus(name = "loadDefaults")]
    fn load_defaults(&self) -> ZResult<()>;

    /// Return whether a family is set for the helper.
    #[zbus(name = "queryFamily")]
    fn query_family(&self, family: &str) -> ZResult<bool>;

    /// Return whether a module is used in the helper.
    #[zbus(name = "queryModule")]
    fn query_module(&self, module: &str) -> ZResult<bool>;

    /// Return whether a port is in the list of ports in the helper.
    #[zbus(name = "queryPort")]
    fn query_port(&self, port: &str, protocol: &str) -> ZResult<bool>;

    /// Remove a non-built-in helper.
    #[zbus(name = "remove")]
    fn remove(&self) -> ZResult<()>;

    /// Permanently remove a port from the helper.
    #[zbus(name = "removePort")]
    fn remove_port(&self, port: &str, protocol: &str) -> ZResult<()>;

    /// Rename a non-built-in helper.
    #[zbus(name = "rename")]
    fn rename(&self, name: &str) -> ZResult<()>;

    /// Permanently set the description of the helper.
    #[zbus(name = "setDescription")]
    fn set_description(&self, description: &str) -> ZResult<()>;

    /// Permanently set the family of the helper.
    #[zbus(name = "setFamily")]
    fn set_family(&self, family: &str) -> ZResult<()>;

    /// Permanently set the module of the helper.
    #[zbus(name = "setModule")]
    fn set_module(&self, module: &str) -> ZResult<()>;

    /// Permanently set the ports of the helper.
    #[zbus(name = "setPorts")]
    fn set_ports(&self, ports: &[(String, String)]) -> ZResult<()>;

    /// Permanently set the short name of the helper.
    #[zbus(name = "setShort")]
    fn set_short(&self, short: &str) -> ZResult<()>;

    /// Permanently set the version of the helper.
    #[zbus(name = "setVersion")]
    fn set_version(&self, version: &str) -> ZResult<()>;

    /// Update the settings of the helper.
    #[zbus(name = "update")]
    fn update(&self, settings: &HelperSettings) -> ZResult<()>;

    /// Signal: emitted when the helper has been removed.
    #[zbus(signal, name = "Removed")]
    fn removed(&self, name: &str) -> ZResult<()>;

    /// Signal: emitted when the helper has been renamed.
    #[zbus(signal, name = "Renamed")]
    fn renamed(&self, name: &str) -> ZResult<()>;

    /// Signal: emitted when the helper has been updated.
    #[zbus(signal, name = "Updated")]
    fn updated(&self, name: &str) -> ZResult<()>;

    /// Property: True if the helper is built-in.
    #[zbus(property)]
    fn builtin(&self) -> ZResult<bool>;

    /// Property: True if a built-in helper has default settings.
    #[zbus(property)]
    fn default(&self) -> ZResult<bool>;

    /// Property: The name of the configuration file.
    #[zbus(property)]
    fn filename(&self) -> ZResult<String>;

    /// Property: The name of the helper.
    #[zbus(property)]
    fn name(&self) -> ZResult<String>;

    /// Property: The path to the configuration directory.
    #[zbus(property)]
    fn path(&self) -> ZResult<String>;
}
