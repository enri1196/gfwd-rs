use std::collections::HashMap;
use zbus::{Connection, Result as ZResult};
use zbus_macros::proxy;

#[proxy(
    interface = "org.fedoraproject.FirewallD1.ipset",
    gen_blocking = false,
    default_service = "org.fedoraproject.FirewallD1",
    default_path = "/org/fedoraproject/FirewallD1"
)]
/// Proxy for the `org.fedoraproject.FirewallD1.ipset` interface.
pub trait IPSet {
    /// Add a new entry to `ipset`. Returns the updated entries.
    #[zbus(name = "addEntry")]
    fn add_entry(&self, ipset: &str, entry: &str) -> ZResult<Vec<String>>;

    /// Get all entries currently in `ipset`.
    #[zbus(name = "getEntries")]
    fn get_entries(&self, ipset: &str) -> ZResult<Vec<String>>;

    /// Get runtime settings of `ipset`:
    /// (version, name, description, type, options, entries).
    #[zbus(name = "getSettings")]
    fn get_settings(
        &self,
        ipset: &str,
    ) -> ZResult<(
        String,                  // version
        String,                  // name
        String,                  // description
        String,                  // type
        HashMap<String, String>, // options
        Vec<String>,             // entries
    )>;

    /// List all runtime ipsets.
    #[zbus(name = "getIPSets")]
    fn list_ipsets(&self) -> ZResult<Vec<String>>;

    /// Return whether `entry` has been added to `ipset`.
    #[zbus(name = "queryService")]
    fn query_entry(&self, ipset: &str, entry: &str) -> ZResult<bool>;

    /// Return whether `ipset` is defined in runtime configuration.
    #[zbus(name = "queryService")]
    fn query_ipset(&self, ipset: &str) -> ZResult<bool>;

    /// Remove `entry` from `ipset`. Returns the updated entries.
    #[zbus(name = "removeEntry")]
    fn remove_entry(&self, ipset: &str, entry: &str) -> ZResult<Vec<String>>;

    /// Replace the entire list of entries in `ipset`.
    #[zbus(name = "setEntries")]
    fn set_entries(&self, entries: &[String]) -> ZResult<()>;

    /// Signal: emitted when an entry has been added to `ipset`.
    #[zbus(signal, name = "EntryAdded")]
    fn entry_added(&self, ipset: &str, entry: &str) -> ZResult<()>;

    /// Signal: emitted when an entry has been removed from `ipset`.
    #[zbus(signal, name = "EntryRemoved")]
    fn entry_removed(&self, ipset: &str, entry: &str) -> ZResult<()>;
}

pub async fn new_ipset_proxy() -> ZResult<IPSetProxy<'static>> {
    let conn = Connection::system().await?;
    IPSetProxy::<'static>::new(&conn).await
}
