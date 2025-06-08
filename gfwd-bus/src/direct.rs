use zbus::Result as ZResult;
use zbus::Connection;
use zbus_macros::proxy;

#[proxy(
    interface = "org.fedoraproject.FirewallD1.direct",
    gen_blocking = false,
    default_service = "org.fedoraproject.FirewallD1",
    default_path = "/org/fedoraproject/FirewallD1"
)]
/// Proxy for the `org.fedoraproject.FirewallD1.direct` interface,
/// which allows runtime manipulation of chains and rules.
pub trait Direct {
    /// Add a new chain to a table for the given IP family (`ipv4`, `ipv6`, or `eb`).
    #[zbus(name = "addChain")]
    fn add_chain(&self, ipv: &str, table: &str, chain: &str) -> ZResult<()>;

    // /// Add a tracked passthrough rule. `args` are the iptables/ip6tables/ebtables arguments.
    // #[zbus(name = "addPassthrough")]
    // fn add_passthrough(&self, ipv: &str, args: &[String]) -> ZResult<()>;

    // /// Add a rule with `args` to `chain` in `table` at `priority`.
    // #[zbus(name = "addRule")]
    // fn add_rule(
    //     &self,
    //     ipv: &str,
    //     table: &str,
    //     chain: &str,
    //     priority: i32,
    //     args: &[String],
    // ) -> ZResult<()>;

    /// Get all chains previously added. Returns `(ipv, table, chain)`.
    #[zbus(name = "getAllChains")]
    fn get_all_chains(&self) -> ZResult<Vec<(String, String, String)>>;

    /// Get all tracked passthrough rules. Returns `(ipv, args)`.
    #[zbus(name = "getAllPassthroughs")]
    fn get_all_passthroughs(&self) -> ZResult<Vec<(String, Vec<String>)>>;

    /// Get all rules previously added. Returns `(ipv, table, chain, priority, args)`.
    #[zbus(name = "getAllRules")]
    fn get_all_rules(&self) -> ZResult<Vec<(String, String, String, i32, Vec<String>)>>;

    /// Get chains in `table` for the given IP family.
    #[zbus(name = "getChains")]
    fn get_chains(&self, ipv: &str, table: &str) -> ZResult<Vec<String>>;

    /// Get tracked passthrough rules for the given IP family.
    #[zbus(name = "getPassthroughs")]
    fn get_passthroughs(&self, ipv: &str) -> ZResult<Vec<Vec<String>>>;

    /// Get rules in `chain` of `table`. Returns `(priority, args)`.
    #[zbus(name = "getRules")]
    fn get_rules(
        &self,
        ipv: &str,
        table: &str,
        chain: &str,
    ) -> ZResult<Vec<(i32, Vec<String>)>>;

    // /// Send an untracked passthrough command. Returns the raw output.
    // #[zbus(name = "passthrough")]
    // fn passthrough(&self, ipv: &str, args: &[String]) -> ZResult<String>;

    /// Return whether a chain exists.
    #[zbus(name = "queryChain")]
    fn query_chain(&self, ipv: &str, table: &str, chain: &str) -> ZResult<bool>;

    // /// Return whether a tracked passthrough rule exists.
    // #[zbus(name = "queryPassthrough")]
    // fn query_passthrough(&self, ipv: &str, args: &[String]) -> ZResult<bool>;

    // /// Return whether a rule exists.
    // #[zbus(name = "queryRule")]
    // fn query_rule(
    //     &self,
    //     ipv: &str,
    //     table: &str,
    //     chain: &str,
    //     priority: i32,
    //     args: &[String],
    // ) -> ZResult<bool>;

    /// Remove all tracked passthrough rules.
    #[zbus(name = "removeAllPassthroughs")]
    fn remove_all_passthroughs(&self) -> ZResult<()>;

    /// Remove a chain.
    #[zbus(name = "removeChain")]
    fn remove_chain(&self, ipv: &str, table: &str, chain: &str) -> ZResult<()>;

    // /// Remove a tracked passthrough rule.
    // #[zbus(name = "removePassthrough")]
    // fn remove_passthrough(&self, ipv: &str, args: &[String]) -> ZResult<()>;

    // /// Remove a rule.
    // #[zbus(name = "removeRule")]
    // fn remove_rule(
    //     &self,
    //     ipv: &str,
    //     table: &str,
    //     chain: &str,
    //     priority: i32,
    //     args: &[String],
    // ) -> ZResult<()>;

    /// Remove all rules from a chain.
    #[zbus(name = "removeRules")]
    fn remove_rules(&self, ipv: &str, table: &str, chain: &str) -> ZResult<()>;

    // --- Signals ---

    /// Emitted when a chain has been added.
    #[zbus(signal, name = "ChainAdded")]
    fn chain_added(&self, ipv: &str, table: &str, chain: &str) -> ZResult<()>;

    /// Emitted when a chain has been removed.
    #[zbus(signal, name = "ChainRemoved")]
    fn chain_removed(&self, ipv: &str, table: &str, chain: &str) -> ZResult<()>;

    // /// Emitted when a tracked passthrough rule has been added.
    // #[zbus(signal, name = "PassthroughAdded")]
    // fn passthrough_added(&self, ipv: &str, args: &[String]) -> ZResult<()>;

    // /// Emitted when a tracked passthrough rule has been removed.
    // #[zbus(signal, name = "PassthroughRemoved")]
    // fn passthrough_removed(&self, ipv: &str, args: &[String]) -> ZResult<()>;

    // /// Emitted when a rule has been added.
    // #[zbus(signal, name = "RuleAdded")]
    // fn rule_added(
    //     &self,
    //     ipv: &str,
    //     table: &str,
    //     chain: &str,
    //     priority: i32,
    //     args: &[String],
    // ) -> ZResult<()>;

    // /// Emitted when a rule has been removed.
    // #[zbus(signal, name = "RuleRemoved")]
    // fn rule_removed(
    //     &self,
    //     ipv: &str,
    //     table: &str,
    //     chain: &str,
    //     priority: i32,
    //     args: &[&str],
    // ) -> ZResult<()>;
}

pub async fn new_firewalld_proxy() -> Result<DirectProxy<'static>, zbus::Error> {
    let conn = Connection::system().await?;
    DirectProxy::<'static>::new(&conn).await
}
