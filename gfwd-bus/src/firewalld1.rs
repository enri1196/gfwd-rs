use std::collections::HashMap;

use zbus::Result as ZResult;
use zbus_macros::proxy;
use zvariant::OwnedValue;

#[proxy(
    interface = "org.fedoraproject.FirewallD1",
    gen_blocking = false,
    default_service = "org.fedoraproject.FirewallD1",
    default_path = "/org/fedoraproject/FirewallD1"
)]
/// Proxy for the `org.fedoraproject.FirewallD1` interface
pub trait FirewallD1 {
    /// Starts the full authorization flow on Firewalld (used by configuration apps).
    #[zbus(name = "authorizeAll")]
    fn authorize_all(&self) -> ZResult<()>;

    /// Performs a full firewall reload (drops state and tears down connections).
    #[zbus(name = "completeReload")]
    fn complete_reload(&self) -> ZResult<()>;

    /// Disables panic mode and returns to the regular policy.
    #[zbus(name = "disablePanicMode")]
    fn disable_panic_mode(&self) -> ZResult<()>;

    /// Enables panic mode, dropping every packet.
    #[zbus(name = "enablePanicMode")]
    fn enable_panic_mode(&self) -> ZResult<()>;

    /// Returns the default zone.
    #[zbus(name = "getDefaultZone")]
    fn get_default_zone(&self) -> ZResult<String>;

    /// Lists all services currently available at runtime.
    #[zbus(name = "listServices")]
    fn list_services(&self) -> ZResult<Vec<String>>;

    /// Returns the key/value settings for a service.
    #[zbus(name = "getServiceSettings2")]
    fn get_service_settings2(&self, service: &str) -> ZResult<HashMap<String, OwnedValue>>;

    /// Reloads rules while keeping state.
    #[zbus(name = "reload")]
    fn reload(&self) -> ZResult<()>;

    /// Converts runtime settings into a permanent configuration.
    #[zbus(name = "runtimeToPermanent")]
    fn runtime_to_permanent(&self) -> ZResult<()>;

    /// Validates the permanent configuration.
    #[zbus(name = "checkPermanentConfig")]
    fn check_permanent_config(&self) -> ZResult<()>;

    /// Sets the default zone (runtime and permanent).
    #[zbus(name = "setDefaultZone")]
    fn set_default_zone(&self, zone: &str) -> ZResult<()>;

    /// Sets the log-denied level (all, unicast, and so on, up to off).
    #[zbus(name = "setLogDenied")]
    fn set_log_denied(&self, value: &str) -> ZResult<()>;

    /// Emitted when the default zone changes.
    #[zbus(signal, name = "DefaultZoneChanged")]
    fn default_zone_changed(&self, zone: &str) -> ZResult<()>;

    /// Emitted when the log-denied level changes.
    #[zbus(signal, name = "LogDeniedChanged")]
    fn log_denied_changed(&self, value: &str) -> ZResult<()>;

    /// Emitted when panic mode is disabled.
    #[zbus(signal, name = "PanicModeDisabled")]
    fn panic_mode_disabled(&self) -> ZResult<()>;

    /// Emitted when panic mode is enabled.
    #[zbus(signal, name = "PanicModeEnabled")]
    fn panic_mode_enabled(&self) -> ZResult<()>;

    /// Emitted on every reload, including complete reloads.
    #[zbus(signal, name = "Reloaded")]
    fn reloaded(&self) -> ZResult<()>;

    /// Indicates whether bridge firewalling is supported.
    #[zbus(property, name = "BRIDGE")]
    fn bridge(&self) -> ZResult<bool>;

    /// Indicates whether ipset support is available.
    #[zbus(property, name = "IPSet")]
    fn ip_set(&self) -> ZResult<bool>;

    /// Returns the list of supported ipset types.
    #[zbus(property, name = "IPSetTypes")]
    fn ip_set_types(&self) -> ZResult<Vec<String>>;

    /// Indicates whether IPv4 firewalling is supported.
    #[zbus(property, name = "IPv4")]
    fn ipv4(&self) -> ZResult<bool>;

    /// Lists the supported IPv4 ICMP types.
    #[zbus(property, name = "IPv4ICMPTypes")]
    fn ipv4_icmp_types(&self) -> ZResult<Vec<String>>;

    /// Indicates whether IPv6 firewalling is supported.
    #[zbus(property, name = "IPv6")]
    fn ipv6(&self) -> ZResult<bool>;

    /// Indicates whether IPv6 reverse-path filtering is enabled.
    #[zbus(property, name = "IPv6_rpfilter")]
    fn ipv6_rpfilter(&self) -> ZResult<bool>;

    /// Lists the supported IPv6 ICMP types.
    #[zbus(property, name = "IPv6ICMPTypes")]
    fn ipv6_icmp_types(&self) -> ZResult<Vec<String>>;
}
