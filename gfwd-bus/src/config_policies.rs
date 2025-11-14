use zbus::Result as ZResult;
use zbus_macros::proxy;

/// Type alias for permanent lockdown whitelist settings.
/// (commands, selinux contexts, users, uids)
pub type LockdownWhitelistSettings = (Vec<String>, Vec<String>, Vec<String>, Vec<i32>);

#[proxy(
    interface = "org.fedoraproject.FirewallD1.config.policies",
    gen_blocking = false,
    default_service = "org.fedoraproject.FirewallD1",
    default_path = "/org/fedoraproject/FirewallD1/config/policies"
)]
/// Proxy for the `org.fedoraproject.FirewallD1.config.policies` interface.
///
/// Interface for permanent lockdown-whitelist configuration. For runtime
/// configuration see `org.fedoraproject.FirewallD1.policies` interface.
pub trait ConfigPolicies {
    /// Add a command to the permanent lockdown whitelist.
    #[zbus(name = "addLockdownWhitelistCommand")]
    fn add_lockdown_whitelist_command(&self, command: &str) -> ZResult<()>;

    /// Add a context to the permanent lockdown whitelist.
    #[zbus(name = "addLockdownWhitelistContext")]
    fn add_lockdown_whitelist_context(&self, context: &str) -> ZResult<()>;

    /// Add a user id to the permanent lockdown whitelist.
    #[zbus(name = "addLockdownWhitelistUid")]
    fn add_lockdown_whitelist_uid(&self, uid: i32) -> ZResult<()>;

    /// Add a user name to the permanent lockdown whitelist.
    #[zbus(name = "addLockdownWhitelistUser")]
    fn add_lockdown_whitelist_user(&self, user: &str) -> ZResult<()>;

    /// Get all settings of the permanent lockdown whitelist.
    #[zbus(name = "getLockdownWhitelist")]
    fn get_lockdown_whitelist(&self) -> ZResult<LockdownWhitelistSettings>;

    /// List all command lines on the permanent lockdown whitelist.
    #[zbus(name = "getLockdownWhitelistCommands")]
    fn get_lockdown_whitelist_commands(&self) -> ZResult<Vec<String>>;

    /// List all contexts on the permanent lockdown whitelist.
    #[zbus(name = "getLockdownWhitelistContexts")]
    fn get_lockdown_whitelist_contexts(&self) -> ZResult<Vec<String>>;

    /// List all user ids on the permanent lockdown whitelist.
    #[zbus(name = "getLockdownWhitelistUids")]
    fn get_lockdown_whitelist_uids(&self) -> ZResult<Vec<i32>>;

    /// List all users on the permanent lockdown whitelist.
    #[zbus(name = "getLockdownWhitelistUsers")]
    fn get_lockdown_whitelist_users(&self) -> ZResult<Vec<String>>;

    /// Query whether a command is on the permanent lockdown whitelist.
    #[zbus(name = "queryLockdownWhitelistCommand")]
    fn query_lockdown_whitelist_command(&self, command: &str) -> ZResult<bool>;

    /// Query whether a context is on the permanent lockdown whitelist.
    #[zbus(name = "queryLockdownWhitelistContext")]
    fn query_lockdown_whitelist_context(&self, context: &str) -> ZResult<bool>;

    /// Query whether a user id is on the permanent lockdown whitelist.
    #[zbus(name = "queryLockdownWhitelistUid")]
    fn query_lockdown_whitelist_uid(&self, uid: i32) -> ZResult<bool>;

    /// Query whether a user is on the permanent lockdown whitelist.
    #[zbus(name = "queryLockdownWhitelistUser")]
    fn query_lockdown_whitelist_user(&self, user: &str) -> ZResult<bool>;

    /// Remove a command from the permanent lockdown whitelist.
    #[zbus(name = "removeLockdownWhitelistCommand")]
    fn remove_lockdown_whitelist_command(&self, command: &str) -> ZResult<()>;

    /// Remove a context from the permanent lockdown whitelist.
    #[zbus(name = "removeLockdownWhitelistContext")]
    fn remove_lockdown_whitelist_context(&self, context: &str) -> ZResult<()>;

    /// Remove a user id from the permanent lockdown whitelist.
    #[zbus(name = "removeLockdownWhitelistUid")]
    fn remove_lockdown_whitelist_uid(&self, uid: i32) -> ZResult<()>;

    /// Remove a user from the permanent lockdown whitelist.
    #[zbus(name = "removeLockdownWhitelistUser")]
    fn remove_lockdown_whitelist_user(&self, user: &str) -> ZResult<()>;

    /// Set the permanent lockdown whitelist configuration.
    #[zbus(name = "setLockdownWhitelist")]
    fn set_lockdown_whitelist(&self, settings: &LockdownWhitelistSettings) -> ZResult<()>;

    /// Signal: emitted when the permanent lockdown whitelist has been updated.
    #[zbus(signal, name = "LockdownWhitelistUpdated")]
    fn lockdown_whitelist_updated(&self) -> ZResult<()>;
}
