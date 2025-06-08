use zbus::{Connection, Result as ZResult, zvariant::ObjectPath};
use zbus_macros::proxy;

#[proxy(
    interface = "org.fedoraproject.FirewallD1.config.icmptype",
    gen_blocking = false,
    default_service = "org.fedoraproject.FirewallD1"
    // default_path is not set because it's dynamic based on the icmptype name
)]
/// Proxy for the `org.fedoraproject.FirewallD1.config.icmptype` interface.
pub trait ConfigIcmpType {
    /// Permanently add a destination ('ipv4' or 'ipv6') to this icmp type.
    /// See the `destination` tag in `firewalld.icmptype(5)`.
    fn add_destination(&self, destination: &str) -> ZResult<()>;

    /// Get the description of the icmp type.
    /// See the `description` tag in `firewalld.icmptype(5)`.
    #[zbus(name = "getDescription")]
    fn get_description(&self) -> ZResult<String>;

    /// Get the list of destinations.
    /// See the `destination` tag in `firewalld.icmptype(5)`.
    #[zbus(name = "getDestinations")]
    fn get_destinations(&self) -> ZResult<Vec<String>>;

    /// Get the permanent settings of the icmp type.
    /// Returns a tuple of (version, name, description, destinations).
    #[zbus(name = "getSettings")]
    fn get_settings(&self) -> ZResult<(String, String, String, Vec<String>)>;

    /// Get the short name of the icmp type.
    /// See the `short` tag in `firewalld.icmptype(5)`.
    #[zbus(name = "getShort")]
    fn get_short(&self) -> ZResult<String>;

    /// Get the version of the icmp type.
    /// See the `version` attribute in `firewalld.icmptype(5)`.
    #[zbus(name = "getVersion")]
    fn get_version(&self) -> ZResult<String>;

    /// Load default settings for a built-in icmp type.
    fn load_defaults(&self) -> ZResult<()>;

    /// Check if a destination ('ipv4' or 'ipv6') is in the list of
    /// destinations.
    fn query_destination(&self, destination: &str) -> ZResult<bool>;

    /// Remove a non-built-in icmp type.
    fn remove(&self) -> ZResult<()>;

    /// Permanently remove a destination ('ipv4' or 'ipv6') from this icmp
    /// type.
    fn remove_destination(&self, destination: &str) -> ZResult<()>;

    /// Rename a non-built-in icmp type.
    fn rename(&self, name: &str) -> ZResult<()>;

    /// Permanently set the description of the icmp type.
    fn set_description(&self, description: &str) -> ZResult<()>;

    /// Permanently set the destinations of the icmp type.
    fn set_destinations(&self, destinations: &[&str]) -> ZResult<()>;

    /// Permanently set the short name of the icmp type.
    fn set_short(&self, short: &str) -> ZResult<()>;

    /// Permanently set the version of the icmp type.
    fn set_version(&self, version: &str) -> ZResult<()>;

    /// Update the permanent settings of the icmp type.
    /// The settings tuple is (version, name, description, destinations).
    #[zbus(name = "update")]
    fn update(&self, settings: (&str, &str, &str, &[&str])) -> ZResult<()>;

    /// Signal: emitted when an icmp type has been removed.
    #[zbus(signal)]
    fn removed(&self, name: &str) -> ZResult<()>;

    /// Signal: emitted when an icmp type has been renamed.
    #[zbus(signal)]
    fn renamed(&self, name: &str) -> ZResult<()>;

    /// Signal: emitted when an icmp type has been updated.
    #[zbus(signal)]
    fn updated(&self, name: &str) -> ZResult<()>;

    /// Property: True if the icmp type is built-in.
    #[zbus(property)]
    fn builtin(&self) -> ZResult<bool>;

    /// Property: True if a built-in icmp type has default settings.
    #[zbus(property, name = "default")]
    fn is_default(&self) -> ZResult<bool>;

    /// Property: The name of the configuration file.
    #[zbus(property)]
    fn filename(&self) -> ZResult<String>;

    /// Property: The name of the icmp type.
    #[zbus(property)]
    fn name(&self) -> ZResult<String>;

    /// Property: The path to the configuration file directory.
    #[zbus(property)]
    fn path(&self) -> ZResult<String>;
}

/// Creates a new proxy for a specific ICMP type configuration.
///
/// # Arguments
///
/// * `icmptype_name` - The name of the ICMP type (e.g., "echo-reply").
pub async fn new_config_icmptype_proxy(
    icmptype_name: &str,
) -> ZResult<ConfigIcmpTypeProxy<'static>> {
    let conn = Connection::system().await?;
    let path_str = format!(
        "/org/fedoraproject/FirewallD1/config/icmptype/{}",
        icmptype_name
    );
    let path = ObjectPath::try_from(path_str)?;

    ConfigIcmpTypeProxy::builder(&conn)
        .path(path)?
        .build()
        .await
}
