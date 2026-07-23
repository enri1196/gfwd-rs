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
    /// Request broad authorization for a firewalld configuration application.
    #[zbus(name = "authorizeAll")]
    fn authorize_all(&self) -> ZResult<()>;

    /// Fully reload the firewall, losing state and terminating connections.
    #[zbus(name = "completeReload")]
    fn complete_reload(&self) -> ZResult<()>;

    /// Disable panic mode.
    #[zbus(name = "disablePanicMode")]
    fn disable_panic_mode(&self) -> ZResult<()>;

    /// Enable panic mode, dropping every packet.
    #[zbus(name = "enablePanicMode")]
    fn enable_panic_mode(&self) -> ZResult<()>;

    /// Return the default zone.
    #[zbus(name = "getDefaultZone")]
    fn get_default_zone(&self) -> ZResult<String>;

    /// List all runtime services.
    #[zbus(name = "listServices")]
    fn list_services(&self) -> ZResult<Vec<String>>;

    /// Return service settings as key/value variants.
    #[zbus(name = "getServiceSettings2")]
    fn get_service_settings2(&self, service: &str) -> ZResult<HashMap<String, OwnedValue>>;

    /// Reload rules while preserving state.
    #[zbus(name = "reload")]
    fn reload(&self) -> ZResult<()>;

    /// Copy runtime configuration to permanent configuration.
    #[zbus(name = "runtimeToPermanent")]
    fn runtime_to_permanent(&self) -> ZResult<()>;

    /// Validate permanent configuration.
    #[zbus(name = "checkPermanentConfig")]
    fn check_permanent_config(&self) -> ZResult<()>;

    /// Set the default zone in runtime and permanent configuration.
    #[zbus(name = "setDefaultZone")]
    fn set_default_zone(&self, zone: &str) -> ZResult<()>;

    /// Set the denied-packet logging level.
    #[zbus(name = "setLogDenied")]
    fn set_log_denied(&self, value: &str) -> ZResult<()>;

    /// Emitted when the default zone changes.
    #[zbus(signal, name = "DefaultZoneChanged")]
    fn default_zone_changed(&self, zone: &str) -> ZResult<()>;

    /// Emitted when `LogDenied` changes.
    #[zbus(signal, name = "LogDeniedChanged")]
    fn log_denied_changed(&self, value: &str) -> ZResult<()>;

    /// Emitted when panic mode is disabled.
    #[zbus(signal, name = "PanicModeDisabled")]
    fn panic_mode_disabled(&self) -> ZResult<()>;

    /// Emitted when panic mode is enabled.
    #[zbus(signal, name = "PanicModeEnabled")]
    fn panic_mode_enabled(&self) -> ZResult<()>;

    /// Emitted for every reload, including a complete reload.
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
