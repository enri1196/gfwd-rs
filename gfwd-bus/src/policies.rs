use zbus::Result as ZResult;
use zbus_macros::proxy;

#[proxy(
    interface = "org.fedoraproject.FirewallD1.policies",
    gen_blocking = false,
    default_service = "org.fedoraproject.FirewallD1",
    default_path = "/org/fedoraproject/FirewallD1"
)]
/// Proxy for the `org.fedoraproject.FirewallD1.policies` interface.
///
/// Enables firewalld to be able to lock down configuration changes from local
/// applications. Local applications or services are able to change the
/// firewall configuration if they are running as root (example: libvirt).
/// With these operations administrator can lock the firewall configuration so
/// that either none or only applications that are in the whitelist are able
/// to request firewall changes.
pub trait Policies {
    /// Add command to the lockdown whitelist.
    #[zbus(name = "addLockdownWhitelistCommand")]
    fn add_lockdown_whitelist_command(&self, command: &str) -> ZResult<()>;

    /// Add context to the lockdown whitelist.
    #[zbus(name = "addLockdownWhitelistContext")]
    fn add_lockdown_whitelist_context(&self, context: &str) -> ZResult<()>;

    /// Add a user id to the lockdown whitelist.
    #[zbus(name = "addLockdownWhitelistUid")]
    fn add_lockdown_whitelist_uid(&self, uid: i32) -> ZResult<()>;

    /// Add a user name to the lockdown whitelist.
    #[zbus(name = "addLockdownWhitelistUser")]
    fn add_lockdown_whitelist_user(&self, user: &str) -> ZResult<()>;

    /// Disable lockdown. This is a runtime and permanent change.
    #[zbus(name = "disableLockdown")]
    fn disable_lockdown(&self) -> ZResult<()>;

    /// Enable lockdown. This is a runtime and permanent change.
    #[zbus(name = "enableLockdown")]
    fn enable_lockdown(&self) -> ZResult<()>;

    /// List all command lines that are on the lockdown whitelist.
    #[zbus(name = "getLockdownWhitelistCommands")]
    fn get_lockdown_whitelist_commands(&self) -> ZResult<Vec<String>>;

    /// List all contexts that are on the lockdown whitelist.
    #[zbus(name = "getLockdownWhitelistContexts")]
    fn get_lockdown_whitelist_contexts(&self) -> ZResult<Vec<String>>;

    /// List all user ids that are on the lockdown whitelist.
    #[zbus(name = "getLockdownWhitelistUids")]
    fn get_lockdown_whitelist_uids(&self) -> ZResult<Vec<i32>>;

    /// List all users that are on the lockdown whitelist.
    #[zbus(name = "getLockdownWhitelistUsers")]
    fn get_lockdown_whitelist_users(&self) -> ZResult<Vec<String>>;

    /// Query whether lockdown is enabled.
    #[zbus(name = "queryLockdown")]
    fn query_lockdown(&self) -> ZResult<bool>;

    /// Query whether a command is on the lockdown whitelist.
    #[zbus(name = "queryLockdownWhitelistCommand")]
    fn query_lockdown_whitelist_command(&self, command: &str) -> ZResult<bool>;

    /// Query whether a context is on the lockdown whitelist.
    #[zbus(name = "queryLockdownWhitelistContext")]
    fn query_lockdown_whitelist_context(&self, context: &str) -> ZResult<bool>;

    /// Query whether a user id is on the lockdown whitelist.
    #[zbus(name = "queryLockdownWhitelistUid")]
    fn query_lockdown_whitelist_uid(&self, uid: i32) -> ZResult<bool>;

    /// Query whether a user is on the lockdown whitelist.
    #[zbus(name = "queryLockdownWhitelistUser")]
    fn query_lockdown_whitelist_user(&self, user: &str) -> ZResult<bool>;

    /// Remove a command from the lockdown whitelist.
    #[zbus(name = "removeLockdownWhitelistCommand")]
    fn remove_lockdown_whitelist_command(&self, command: &str) -> ZResult<()>;

    /// Remove a context from the lockdown whitelist.
    #[zbus(name = "removeLockdownWhitelistContext")]
    fn remove_lockdown_whitelist_context(&self, context: &str) -> ZResult<()>;

    /// Remove a user id from the lockdown whitelist.
    #[zbus(name = "removeLockdownWhitelistUid")]
    fn remove_lockdown_whitelist_uid(&self, uid: i32) -> ZResult<()>;

    /// Remove a user from the lockdown whitelist.
    #[zbus(name = "removeLockdownWhitelistUser")]
    fn remove_lockdown_whitelist_user(&self, user: &str) -> ZResult<()>;

    /// Signal: emitted when lockdown has been disabled.
    #[zbus(signal, name = "LockdownDisabled")]
    fn lockdown_disabled(&self) -> ZResult<()>;

    /// Signal: emitted when lockdown has been enabled.
    #[zbus(signal, name = "LockdownEnabled")]
    fn lockdown_enabled(&self) -> ZResult<()>;

    /// Signal: emitted when a command has been added to the whitelist.
    #[zbus(signal, name = "LockdownWhitelistCommandAdded")]
    fn lockdown_whitelist_command_added(&self, command: &str) -> ZResult<()>;

    /// Signal: emitted when a command has been removed from the whitelist.
    #[zbus(signal, name = "LockdownWhitelistCommandRemoved")]
    fn lockdown_whitelist_command_removed(&self, command: &str) -> ZResult<()>;

    /// Signal: emitted when a context has been added to the whitelist.
    #[zbus(signal, name = "LockdownWhitelistContextAdded")]
    fn lockdown_whitelist_context_added(&self, context: &str) -> ZResult<()>;

    /// Signal: emitted when a context has been removed from the whitelist.
    #[zbus(signal, name = "LockdownWhitelistContextRemoved")]
    fn lockdown_whitelist_context_removed(&self, context: &str) -> ZResult<()>;

    /// Signal: emitted when a user id has been added to the whitelist.
    #[zbus(signal, name = "LockdownWhitelistUidAdded")]
    fn lockdown_whitelist_uid_added(&self, uid: i32) -> ZResult<()>;

    /// Signal: emitted when a user id has been removed from the whitelist.
    #[zbus(signal, name = "LockdownWhitelistUidRemoved")]
    fn lockdown_whitelist_uid_removed(&self, uid: i32) -> ZResult<()>;

    /// Signal: emitted when a user has been added to the whitelist.
    #[zbus(signal, name = "LockdownWhitelistUserAdded")]
    fn lockdown_whitelist_user_added(&self, user: &str) -> ZResult<()>;

    /// Signal: emitted when a user has been removed from the whitelist.
    #[zbus(signal, name = "LockdownWhitelistUserRemoved")]
    fn lockdown_whitelist_user_removed(&self, user: &str) -> ZResult<()>;
}
