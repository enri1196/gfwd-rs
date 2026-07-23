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

/// Result of asking the refresh coordinator to handle a configuration event.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RefreshRequest {
    /// A refresh may start immediately.
    Start,
    /// The event was folded into one pending follow-up refresh.
    Coalesced,
}

/// Pure request identity, refresh coalescing, and watcher-warning state.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ConfigurationRefreshCoordinator {
    generation: u64,
    refresh_in_flight: bool,
    refresh_pending: bool,
    watch_warning: Option<String>,
}

impl ConfigurationRefreshCoordinator {
    /// Advance and return the identity for a newly selected or reloaded zone.
    pub fn selection_changed(&mut self) -> u64 {
        self.generation = self.generation.wrapping_add(1);
        self.generation
    }

    /// Return the current selected-zone request identity.
    pub fn generation(&self) -> u64 {
        self.generation
    }

    /// Return whether a response belongs to the current selected-zone request.
    pub fn accepts(&self, generation: u64) -> bool {
        self.generation == generation
    }

    /// Start a refresh or coalesce it when work is blocked or already active.
    pub fn request_refresh(&mut self, blocked: bool) -> RefreshRequest {
        if blocked || self.refresh_in_flight {
            self.refresh_pending = true;
            RefreshRequest::Coalesced
        } else {
            self.refresh_in_flight = true;
            RefreshRequest::Start
        }
    }

    /// Finish active work and return whether one follow-up refresh must start.
    pub fn finish_refresh(&mut self) -> bool {
        self.refresh_in_flight = false;
        if self.refresh_pending {
            self.refresh_pending = false;
            self.refresh_in_flight = true;
            true
        } else {
            false
        }
    }

    /// Return whether a refresh is currently active.
    pub fn is_refreshing(&self) -> bool {
        self.refresh_in_flight
    }

    /// Return whether at least one event is waiting for a follow-up refresh.
    pub fn has_pending(&self) -> bool {
        self.refresh_pending
    }

    /// Retain one actionable warning for a failed watcher.
    pub fn watcher_failed(&mut self, message: String) {
        self.watch_warning = Some(message);
    }

    /// Clear a prior watcher failure after the stream produces an event.
    pub fn watcher_recovered(&mut self) {
        self.watch_warning = None;
    }

    /// Return the current watcher warning, if any.
    pub fn watch_warning(&self) -> Option<&str> {
        self.watch_warning.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::{ConfigurationRefreshCoordinator, RefreshRequest};

    #[test]
    fn selection_changes_reject_stale_responses() {
        let mut coordinator = ConfigurationRefreshCoordinator::default();
        let first = coordinator.selection_changed();
        let second = coordinator.selection_changed();

        assert!(!coordinator.accepts(first));
        assert!(coordinator.accepts(second));
    }

    #[test]
    fn pending_actions_defer_refresh_until_unblocked() {
        let mut coordinator = ConfigurationRefreshCoordinator::default();

        assert_eq!(coordinator.request_refresh(true), RefreshRequest::Coalesced);
        assert!(coordinator.has_pending());
        assert!(coordinator.finish_refresh());
        assert!(coordinator.is_refreshing());
    }

    #[test]
    fn signal_bursts_coalesce_to_one_follow_up() {
        let mut coordinator = ConfigurationRefreshCoordinator::default();

        assert_eq!(coordinator.request_refresh(false), RefreshRequest::Start);
        assert_eq!(
            coordinator.request_refresh(false),
            RefreshRequest::Coalesced
        );
        assert_eq!(
            coordinator.request_refresh(false),
            RefreshRequest::Coalesced
        );
        assert!(coordinator.finish_refresh());
        assert!(!coordinator.finish_refresh());
    }

    #[test]
    fn successful_refresh_returns_to_idle() {
        let mut coordinator = ConfigurationRefreshCoordinator::default();
        assert_eq!(coordinator.request_refresh(false), RefreshRequest::Start);

        assert!(!coordinator.finish_refresh());
        assert!(!coordinator.is_refreshing());
        assert!(!coordinator.has_pending());
    }

    #[test]
    fn failed_refresh_can_recover_with_queued_work() {
        let mut coordinator = ConfigurationRefreshCoordinator::default();
        assert_eq!(coordinator.request_refresh(false), RefreshRequest::Start);
        assert_eq!(
            coordinator.request_refresh(false),
            RefreshRequest::Coalesced
        );

        assert!(coordinator.finish_refresh());
        assert!(!coordinator.finish_refresh());
        assert!(!coordinator.is_refreshing());
    }

    #[test]
    fn watcher_warning_clears_after_recovery() {
        let mut coordinator = ConfigurationRefreshCoordinator::default();
        coordinator.watcher_failed("permission denied".into());
        assert_eq!(coordinator.watch_warning(), Some("permission denied"));

        coordinator.watcher_recovered();
        assert_eq!(coordinator.watch_warning(), None);
    }
}
