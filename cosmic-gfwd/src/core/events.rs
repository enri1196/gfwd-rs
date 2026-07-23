/// A firewalld configuration event relevant to the selected-zone workflow.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConfigurationEvent {
    /// Firewalld completed a global reload.
    Reloaded,
    /// A runtime signal changed the selected zone.
    RuntimeZoneChanged {
        /// Selected zone named by the signal.
        zone: String,
    },
    /// The selected permanent zone was updated.
    PermanentZoneUpdated {
        /// Selected zone object being watched.
        zone: String,
    },
    /// The selected permanent zone was removed.
    PermanentZoneRemoved {
        /// Removed zone.
        zone: String,
    },
    /// The selected permanent zone was renamed.
    PermanentZoneRenamed {
        /// Previous selected zone name.
        old_zone: String,
        /// New zone name emitted by firewalld.
        new_zone: String,
    },
}
