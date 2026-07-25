use crate::{core::BrokerError, models::IpSetDetails, ui::IpSetViewAction};

/// Messages owned by the IP-set feature slice.
#[derive(Clone, Debug)]
pub(crate) enum Message {
    /// Handle an action emitted by the IP-set view.
    View(IpSetViewAction),
    /// The permanent IP-set list finished loading.
    ListLoaded(Result<Vec<String>, BrokerError>),
    /// A selected IP-set detail request finished.
    DetailsLoaded {
        /// IP set requested by the asynchronous task.
        ipset_name: String,
        /// Loaded details or the broker failure.
        result: Result<IpSetDetails, BrokerError>,
    },
    /// Adding an entry to an IP set finished.
    EntryAdded {
        /// Mutated IP set.
        ipset_name: String,
        /// Mutation result.
        result: Result<(), BrokerError>,
    },
    /// Removing an entry from an IP set finished.
    EntryRemoved {
        /// Mutated IP set.
        ipset_name: String,
        /// Mutation result.
        result: Result<(), BrokerError>,
    },
    /// Creating an IP set finished.
    Created {
        /// Requested IP-set name.
        ipset_name: String,
        /// Mutation result.
        result: Result<(), BrokerError>,
    },
    /// Deleting an IP set finished.
    Deleted {
        /// Deleted IP-set name.
        ipset_name: String,
        /// Mutation result.
        result: Result<(), BrokerError>,
    },
}
