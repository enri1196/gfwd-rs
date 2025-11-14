use std::collections::HashMap;
use zbus::Result as ZResult;
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

#[proxy(
    interface = "org.fedoraproject.FirewallD1.config.ipset",
    gen_blocking = false,
    default_service = "org.fedoraproject.FirewallD1"
)]
/// Proxy for the `org.fedoraproject.FirewallD1.config.ipset` interface.
///
/// Interface for permanent ipset configuration.
pub trait ConfigIPSet {
    /// Permanently add an entry to the list of entries for the IPSet.
    #[zbus(name = "addEntry")]
    fn add_entry(&self, entry: &str) -> ZResult<()>;

    /// Permanently add an option (key, value) to the IPSet.
    #[zbus(name = "addOption")]
    fn add_option(&self, key: &str, value: &str) -> ZResult<()>;

    /// Get the description of the IPSet.
    #[zbus(name = "getDescription")]
    fn get_description(&self) -> ZResult<String>;

    /// Get the list of entries added to the IPSet.
    #[zbus(name = "getEntries")]
    fn get_entries(&self) -> ZResult<Vec<String>>;

    /// Get the dictionary of options set for the IPSet.
    #[zbus(name = "getOptions")]
    fn get_options(&self) -> ZResult<HashMap<String, String>>;

    /// Get the permanent settings of the IPSet.
    #[zbus(name = "getSettings")]
    fn get_settings(&self) -> ZResult<IPSetSettings>;

    /// Get the short name of the IPSet.
    #[zbus(name = "getShort")]
    fn get_short(&self) -> ZResult<String>;

    /// Get the type of the IPSet.
    #[zbus(name = "getType")]
    fn get_type(&self) -> ZResult<String>;

    /// Get the version of the IPSet.
    #[zbus(name = "getVersion")]
    fn get_version(&self) -> ZResult<String>;

    /// Load default settings for a built-in IPSet.
    #[zbus(name = "loadDefaults")]
    fn load_defaults(&self) -> ZResult<()>;

    /// Return whether an entry has been added to the IPSet.
    #[zbus(name = "queryEntry")]
    fn query_entry(&self, entry: &str) -> ZResult<bool>;

    /// Return whether an option (key, value) has been added to the IPSet.
    #[zbus(name = "queryOption")]
    fn query_option(&self, key: &str, value: &str) -> ZResult<bool>;

    /// Remove a non-built-in IPSet.
    #[zbus(name = "remove")]
    fn remove(&self) -> ZResult<()>;

    /// Permanently remove an entry from the IPSet.
    #[zbus(name = "removeEntry")]
    fn remove_entry(&self, entry: &str) -> ZResult<()>;

    /// Permanently remove an option by its key from the IPSet.
    #[zbus(name = "removeOption")]
    fn remove_option(&self, key: &str) -> ZResult<()>;

    /// Rename a non-built-in IPSet.
    #[zbus(name = "rename")]
    fn rename(&self, name: &str) -> ZResult<()>;

    /// Permanently set the description of the IPSet.
    #[zbus(name = "setDescription")]
    fn set_description(&self, description: &str) -> ZResult<()>;

    /// Permanently set the list of entries for the IPSet.
    #[zbus(name = "setEntries")]
    fn set_entries(&self, entries: &[String]) -> ZResult<()>;

    /// Permanently set the dictionary of options for the IPSet.
    #[zbus(name = "setOptions")]
    fn set_options(&self, options: &HashMap<String, String>) -> ZResult<()>;

    /// Permanently set the short name of the IPSet.
    #[zbus(name = "setShort")]
    fn set_short(&self, short: &str) -> ZResult<()>;

    /// Permanently set the type of the IPSet.
    #[zbus(name = "setType")]
    fn set_type(&self, ipset_type: &str) -> ZResult<()>;

    /// Permanently set the version of the IPSet.
    #[zbus(name = "setVersion")]
    fn set_version(&self, version: &str) -> ZResult<()>;

    /// Update the settings of the IPSet.
    #[zbus(name = "update")]
    fn update(&self, settings: &IPSetSettings) -> ZResult<()>;

    /// Signal: emitted when the IPSet has been removed.
    #[zbus(signal, name = "Removed")]
    fn removed(&self, name: &str) -> ZResult<()>;

    /// Signal: emitted when the IPSet has been renamed.
    #[zbus(signal, name = "Renamed")]
    fn renamed(&self, name: &str) -> ZResult<()>;

    /// Signal: emitted when the IPSet has been updated.
    #[zbus(signal, name = "Updated")]
    fn updated(&self, name: &str) -> ZResult<()>;

    /// Property: True if the IPSet is built-in.
    #[zbus(property)]
    fn builtin(&self) -> ZResult<bool>;

    /// Property: True if a built-in IPSet has default settings.
    #[zbus(property)]
    fn default(&self) -> ZResult<bool>;

    /// Property: The name of the configuration file.
    #[zbus(property)]
    fn filename(&self) -> ZResult<String>;

    /// Property: The name of the IPSet.
    #[zbus(property)]
    fn name(&self) -> ZResult<String>;

    /// Property: The path to the configuration directory.
    #[zbus(property)]
    fn path(&self) -> ZResult<String>;
}
