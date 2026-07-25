use std::collections::HashSet;

use crate::{
    core::{BrokerError, FirewalldStatus},
    models::ZoneDetails,
    ui::ZoneViewAction,
};

/// Messages owned by the zone and ordinary firewalld-control slice.
#[derive(Clone, Debug)]
pub(crate) enum Message {
    /// Handle an action emitted by the selected-zone view.
    View(ZoneViewAction),
    /// The permanent zone list finished loading.
    ListLoaded(Result<Vec<String>, BrokerError>),
    /// A selected-zone detail request finished.
    DetailsLoaded {
        /// Zone requested by the asynchronous task.
        zone_name: String,
        /// Loaded details or the broker failure.
        result: Box<Result<ZoneDetails, BrokerError>>,
    },
    /// The configured default zone finished loading.
    DefaultLoaded(Result<String, BrokerError>),
    /// The active runtime zones finished loading.
    ActiveLoaded(Result<HashSet<String>, BrokerError>),
    /// Changing the default zone finished.
    DefaultSet(Result<(), BrokerError>),
    /// Creating a zone finished.
    Created {
        /// Requested zone name.
        zone_name: String,
        /// Mutation result.
        result: Result<(), BrokerError>,
    },
    /// Deleting a zone finished.
    Deleted {
        /// Deleted zone name.
        zone_name: String,
        /// Mutation result.
        result: Result<(), BrokerError>,
    },
    /// Adding an item to a zone finished.
    ItemAdded {
        /// Mutated zone.
        zone_name: String,
        /// Mutation result.
        result: Result<(), BrokerError>,
    },
    /// Removing an item from a zone finished.
    ItemRemoved {
        /// Mutated zone.
        zone_name: String,
        /// Mutation result.
        result: Result<(), BrokerError>,
    },
    /// The ordinary firewalld status request finished.
    FirewalldStatusLoaded(Result<FirewalldStatus, BrokerError>),
    /// An ordinary daemon start or stop request finished.
    DaemonControlFinished(Result<(), BrokerError>),
}
