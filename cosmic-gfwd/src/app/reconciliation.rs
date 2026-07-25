//! Selected-zone reconciliation lifecycle and refresh coordination.
//!
//! The controller in this module owns request identity, stale-response
//! rejection, event coalescing, watcher health, and the independently loaded
//! runtime/permanent comparison. It intentionally has no dependency on COSMIC
//! widgets, tasks, or localization.

use crate::core::{
    ConfigurationRefreshCoordinator, RefreshRequest, ZoneReconciliationData,
    ZoneReconciliationState,
};

/// Reconciliation-specific messages delivered to the application.
#[derive(Clone, Debug)]
pub(crate) enum Message {
    /// A selected-zone permanent/runtime comparison completed.
    LoadCompleted {
        /// Zone requested by the asynchronous task.
        zone: String,
        /// Selection generation captured when the request began.
        generation: u64,
        /// Loaded reconciliation data or a displayable broker error.
        result: Box<Result<ZoneReconciliationData, String>>,
    },
    /// A broker configuration event or watcher failure was received.
    ConfigurationEvent(Result<crate::core::ConfigurationEvent, String>),
    /// Applying permanent configuration to the global runtime finished.
    PermanentApplied(Result<(), crate::core::BrokerError>),
    /// Persisting the global runtime configuration permanently finished.
    RuntimePersisted(Result<(), crate::core::BrokerError>),
}

/// Owns selected-zone reconciliation state and pure refresh coordination.
#[derive(Debug, Default)]
pub(crate) struct State {
    state: ZoneReconciliationState,
    coordinator: ConfigurationRefreshCoordinator,
}

impl State {
    /// Return the current independently loaded reconciliation state.
    pub(crate) fn state(&self) -> &ZoneReconciliationState {
        &self.state
    }

    /// Return the selected zone represented by the reconciliation state.
    pub(crate) fn selected_zone(&self) -> Option<&str> {
        match &self.state {
            ZoneReconciliationState::Loading { zone }
            | ZoneReconciliationState::InSync { zone, .. }
            | ZoneReconciliationState::Different { zone, .. }
            | ZoneReconciliationState::Incomplete { zone, .. }
            | ZoneReconciliationState::Error { zone, .. } => Some(zone),
            ZoneReconciliationState::Unavailable { zone } => zone.as_deref(),
        }
    }

    /// Rebind reconciliation to a newly selected or renamed permanent zone.
    pub(crate) fn selection_changed(&mut self, zone: Option<String>) -> u64 {
        let generation = self.coordinator.selection_changed();
        self.state = ZoneReconciliationState::Unavailable { zone };
        generation
    }

    /// Mark runtime comparison unavailable without invalidating permanent data.
    pub(crate) fn set_unavailable(&mut self, zone: Option<String>) {
        self.state = ZoneReconciliationState::Unavailable { zone };
    }

    /// Begin a selected-zone comparison and return its request generation.
    pub(crate) fn begin_load(&mut self, zone: String) -> u64 {
        self.state = ZoneReconciliationState::Loading { zone };
        self.coordinator.generation()
    }

    /// Complete a load only when both its zone and generation are current.
    ///
    /// Returns `true` when the result was accepted and applied.
    pub(crate) fn complete_load(
        &mut self,
        zone: String,
        generation: u64,
        result: Result<ZoneReconciliationData, String>,
    ) -> bool {
        if !self.coordinator.accepts(generation) || self.selected_zone() != Some(zone.as_str()) {
            return false;
        }

        self.state = match result {
            Ok(data) => ZoneReconciliationState::from_data(zone, data),
            Err(message) => ZoneReconciliationState::Error { zone, message },
        };
        true
    }

    /// Schedule a refresh after a successfully received configuration event.
    pub(crate) fn handle_configuration_event(&mut self, blocked: bool) -> RefreshRequest {
        self.coordinator.watcher_recovered();
        self.coordinator.request_refresh(blocked)
    }

    /// Retain an actionable watcher warning while manual refresh remains usable.
    pub(crate) fn watcher_failed(&mut self, message: String) {
        self.coordinator.watcher_failed(message);
    }

    /// Return the current watcher warning, if any.
    pub(crate) fn watch_warning(&self) -> Option<&str> {
        self.coordinator.watch_warning()
    }

    /// Finish active refresh work and report whether one coalesced follow-up is due.
    pub(crate) fn refresh_finished(&mut self) -> bool {
        self.coordinator.finish_refresh()
    }

    /// Return whether coordinated refresh work is currently active.
    pub(crate) fn is_refreshing(&self) -> bool {
        self.coordinator.is_refreshing()
    }

    /// Consume a deferred refresh when no refresh is currently active.
    pub(crate) fn take_deferred_refresh(&mut self) -> bool {
        self.coordinator.has_pending()
            && !self.coordinator.is_refreshing()
            && self.coordinator.finish_refresh()
    }

    /// Return whether a reconciliation comparison is currently loading.
    pub(crate) fn is_loading(&self) -> bool {
        matches!(self.state, ZoneReconciliationState::Loading { .. })
    }
}

#[cfg(test)]
mod tests {
    use crate::core::{
        ComparisonCompleteness, RefreshRequest, ZoneReconciliationData, ZoneReconciliationState,
        reconciliation::{ZoneReconciliation, ZoneSettingsSnapshot},
    };

    use super::State;

    fn in_sync_data() -> ZoneReconciliationData {
        ZoneReconciliationData {
            permanent: ZoneSettingsSnapshot::default(),
            runtime: ZoneSettingsSnapshot::default(),
            reconciliation: ZoneReconciliation {
                differences: Vec::new(),
                completeness: ComparisonCompleteness::Complete,
            },
        }
    }

    #[test]
    fn selection_changes_reset_state_and_advance_identity() {
        let mut controller = State::default();

        let first = controller.selection_changed(Some("public".into()));
        let second = controller.selection_changed(Some("work".into()));

        assert_ne!(first, second);
        assert_eq!(controller.selected_zone(), Some("work"));
        assert!(matches!(
            controller.state(),
            ZoneReconciliationState::Unavailable { .. }
        ));
    }

    #[test]
    fn stale_zone_and_generation_responses_are_ignored() {
        let mut controller = State::default();
        controller.selection_changed(Some("public".into()));
        let stale_generation = controller.begin_load("public".into());
        controller.selection_changed(Some("work".into()));
        let current_generation = controller.begin_load("work".into());

        assert!(!controller.complete_load("public".into(), stale_generation, Ok(in_sync_data()),));
        assert!(
            !controller.complete_load("public".into(), current_generation, Ok(in_sync_data()),)
        );
        assert_eq!(controller.selected_zone(), Some("work"));
    }

    #[test]
    fn event_burst_during_load_schedules_one_follow_up() {
        let mut controller = State::default();
        controller.selection_changed(Some("public".into()));
        controller.begin_load("public".into());

        assert_eq!(
            controller.handle_configuration_event(true),
            RefreshRequest::Coalesced
        );
        assert_eq!(
            controller.handle_configuration_event(true),
            RefreshRequest::Coalesced
        );
        assert!(controller.refresh_finished());
        assert!(!controller.refresh_finished());
    }

    #[test]
    fn successful_refresh_can_return_to_idle() {
        let mut controller = State::default();
        assert_eq!(
            controller.handle_configuration_event(false),
            RefreshRequest::Start
        );

        assert!(!controller.refresh_finished());
    }

    #[test]
    fn failed_refresh_can_recover_with_queued_work() {
        let mut controller = State::default();
        assert_eq!(
            controller.handle_configuration_event(false),
            RefreshRequest::Start
        );
        assert_eq!(
            controller.handle_configuration_event(false),
            RefreshRequest::Coalesced
        );

        assert!(controller.refresh_finished());
        assert!(!controller.refresh_finished());
    }

    #[test]
    fn watcher_failure_and_recovery_preserve_manual_state() {
        let mut controller = State::default();
        controller.selection_changed(Some("public".into()));
        controller.watcher_failed("permission denied".into());

        assert_eq!(controller.watch_warning(), Some("permission denied"));
        assert_eq!(controller.selected_zone(), Some("public"));

        controller.handle_configuration_event(false);
        assert_eq!(controller.watch_warning(), None);
    }

    #[test]
    fn successful_and_failed_loads_preserve_selected_zone() {
        let mut controller = State::default();
        controller.selection_changed(Some("public".into()));
        let generation = controller.begin_load("public".into());
        assert!(controller.complete_load("public".into(), generation, Ok(in_sync_data()),));
        assert!(matches!(
            controller.state(),
            ZoneReconciliationState::InSync { .. }
        ));

        let generation = controller.begin_load("public".into());
        assert!(controller.complete_load(
            "public".into(),
            generation,
            Err("runtime unavailable".into()),
        ));
        assert!(matches!(
            controller.state(),
            ZoneReconciliationState::Error { .. }
        ));
        assert_eq!(controller.selected_zone(), Some("public"));
    }

    #[test]
    fn permanent_zone_rename_rebinds_selection() {
        let mut controller = State::default();
        controller.selection_changed(Some("old".into()));

        controller.selection_changed(Some("new".into()));

        assert_eq!(controller.selected_zone(), Some("new"));
    }
}
