use zbus::{Connection, Result as ZResult};
use zbus_macros::proxy;

/// Type alias for a direct chain: (ipv, table, chain).
pub type DirectChain = (String, String, String);

/// Type alias for a direct rule: (ipv, table, chain, priority, args).
pub type DirectRule = (String, String, String, i32, Vec<String>);

/// Type alias for a direct passthrough rule: (ipv, args).
pub type DirectPassthrough = (String, Vec<String>);

/// Type alias for direct rule details: (priority, args).
pub type DirectRuleDetails = (i32, Vec<String>);

/// Type alias for all direct settings: (chains, rules, passthroughs).
pub type DirectSettings =
    (Vec<DirectChain>, Vec<DirectRule>, Vec<DirectPassthrough>);

#[proxy(
    interface = "org.fedoraproject.FirewallD1.config.direct",
    gen_blocking = false,
    default_service = "org.fedoraproject.FirewallD1",
    default_path = "/org/fedoraproject/FirewallD1/config/direct"
)]
/// Proxy for the `org.fedoraproject.FirewallD1.config.direct` interface.
///
/// Interface for permanent direct configuration. For runtime direct
/// configuration see `org.fedoraproject.FirewallD1.direct` interface.
pub trait ConfigDirect {
    /// Add a new chain to a table.
    #[zbus(name = "addChain")]
    fn add_chain(
        &self,
        ipv: &str,
        table: &str,
        chain: &str,
    ) -> ZResult<()>;

    /// Add a passthrough rule.
    #[zbus(name = "addPassthrough")]
    fn add_passthrough(&self, ipv: &str, args: &[String]) -> ZResult<()>;

    /// Add a rule to a chain in a table with a given priority.
    #[zbus(name = "addRule")]
    fn add_rule(
        &self,
        ipv: &str,
        table: &str,
        chain: &str,
        priority: i32,
        args: &[String],
    ) -> ZResult<()>;

    /// Get all chains added to all tables.
    #[zbus(name = "getAllChains")]
    fn get_all_chains(&self) -> ZResult<Vec<DirectChain>>;

    /// Get all passthrough rules.
    #[zbus(name = "getAllPassthroughs")]
    fn get_all_passthroughs(&self) -> ZResult<Vec<DirectPassthrough>>;

    /// Get all rules added to all chains in all tables.
    #[zbus(name = "getAllRules")]
    fn get_all_rules(&self) -> ZResult<Vec<DirectRule>>;

    /// Get an array of chains added to a table.
    #[zbus(name = "getChains")]
    fn get_chains(&self, ipv: &str, table: &str) -> ZResult<Vec<String>>;

    /// Get tracked passthrough rules.
    #[zbus(name = "getPassthroughs")]
    fn get_passthroughs(&self, ipv: &str) -> ZResult<Vec<Vec<String>>>;

    /// Get all rules added to a chain in a table.
    #[zbus(name = "getRules")]
    fn get_rules(
        &self,
        ipv: &str,
        table: &str,
        chain: &str,
    ) -> ZResult<Vec<DirectRuleDetails>>;

    /// Get all settings of the permanent direct configuration.
    #[zbus(name = "getSettings")]
    fn get_settings(&self) -> ZResult<DirectSettings>;

    /// Return whether a chain exists in a table.
    #[zbus(name = "queryChain")]
    fn query_chain(
        &self,
        ipv: &str,
        table: &str,
        chain: &str,
    ) -> ZResult<bool>;

    /// Return whether a tracked passthrough rule exists.
    #[zbus(name = "queryPassthrough")]
    fn query_passthrough(&self, ipv: &str, args: &[String]) -> ZResult<bool>;

    /// Return whether a rule exists in a chain in a table.
    #[zbus(name = "queryRule")]
    fn query_rule(
        &self,
        ipv: &str,
        table: &str,
        chain: &str,
        priority: i32,
        args: &[String],
    ) -> ZResult<bool>;

    /// Remove a chain from a table.
    #[zbus(name = "removeChain")]
    fn remove_chain(
        &self,
        ipv: &str,
        table: &str,
        chain: &str,
    ) -> ZResult<()>;

    /// Remove a passthrough rule.
    #[zbus(name = "removePassthrough")]
    fn remove_passthrough(&self, ipv: &str, args: &[String]) -> ZResult<()>;

    /// Remove a rule from a chain in a table.
    #[zbus(name = "removeRule")]
    fn remove_rule(
        &self,
        ipv: &str,
        table: &str,
        chain: &str,
        priority: i32,
        args: &[String],
    ) -> ZResult<()>;

    /// Remove all rules from a chain in a table.
    #[zbus(name = "removeRules")]
    fn remove_rules(
        &self,
        ipv: &str,
        table: &str,
        chain: &str,
    ) -> ZResult<()>;

    /// Update the permanent direct configuration with the given settings.
    #[zbus(name = "update")]
    fn update(&self, settings: &DirectSettings) -> ZResult<()>;

    /// Signal: emitted when the configuration has been updated.
    #[zbus(signal, name = "Updated")]
    fn updated(&self) -> ZResult<()>;
}

pub async fn new_config_direct_proxy() -> ZResult<ConfigDirectProxy<'static>> {
    let conn = Connection::system().await?;
    ConfigDirectProxy::<'static>::new(&conn).await
}
