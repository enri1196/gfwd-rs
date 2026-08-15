//! Selected-zone reconciliation lifecycle and refresh coordination.
//!
//! The controller in this module owns request identity, stale-response
//! rejection, event coalescing, watcher health, and the independently loaded
//! runtime/permanent comparison. It intentionally has no dependency on COSMIC
//! widgets, tasks, or localization.

mod model;
mod view;

pub(crate) use model::{ReconciliationPresentation, ReconciliationPresentationStatus};
pub(crate) use view::{ReconciliationAction, reconciliation_drawer};

use crate::core::{
    BrokerError, ConfigurationEvent, ConfigurationRefreshCoordinator, FwdBroker, RefreshRequest,
    ZoneReconciliationData, ZoneReconciliationState,
};
use cosmic::Task;
use futures_util::{StreamExt, stream::BoxStream};

use super::outcome::Outcome;

/// Reconciliation-specific messages delivered to the application.
#[derive(Clone, Debug)]
pub(crate) enum Message {
    /// Handle an action shared by the banner and review drawer.
    Action(ReconciliationAction),
    /// Begin loading the selected-zone comparison.
    Load(String),
    /// Apply permanent configuration after root confirmation.
    ApplyPermanent,
    /// Persist runtime configuration after root confirmation.
    PersistRuntime,
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

/// Immutable root projection needed by the reconciliation reducer.
pub(crate) struct Context<'a> {
    /// Zone selected by navigation.
    pub(crate) selected_zone: Option<&'a str>,
    /// Zone represented by the ready detail view.
    pub(crate) ready_zone: Option<&'a str>,
    /// Whether ordinary firewalld status is active.
    pub(crate) firewalld_active: bool,
    /// Whether any slice owns the global mutation slot.
    pub(crate) mutation_pending: bool,
}

/// Reconciliation-owned asynchronous work.
#[derive(Clone, Debug)]
pub(crate) enum Effect {
    /// Load permanent and runtime snapshots.
    Load { zone: String, generation: u64 },
    /// Apply permanent configuration globally to runtime.
    ApplyPermanent,
    /// Persist global runtime configuration permanently.
    PersistRuntime,
}

/// Neutral mutation kinds interpreted by root localization.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Mutation {
    ApplyPermanent,
    PersistRuntime,
}

/// Root coordination emitted by reconciliation.
#[derive(Debug)]
pub(crate) enum Request {
    OpenReview,
    ConfirmApplyPermanent,
    ConfirmPersistRuntime,
    BeginMutation(Mutation),
    FinishMutation(Result<(), BrokerError>),
    ClearRuntimeDirty,
    ConfigurationRefresh(ConfigurationEvent),
    RefreshFirewalldStatus,
    RefreshZones,
    RefreshIpSets,
    RefreshCatalogs,
}

/// Owns selected-zone reconciliation state and pure refresh coordination.
#[derive(Debug, Default)]
pub(crate) struct State {
    last_checked: Option<std::time::SystemTime>,
    state: ZoneReconciliationState,
    coordinator: ConfigurationRefreshCoordinator,
}

impl State {
    pub(crate) fn last_checked_age_seconds(&self) -> Option<u64> {
        self.last_checked
            .and_then(|checked| checked.elapsed().ok())
            .map(|elapsed| elapsed.as_secs())
    }
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

        let succeeded = result.is_ok();
        self.state = match result {
            Ok(data) => ZoneReconciliationState::from_data(zone, data),
            Err(message) => ZoneReconciliationState::Error { zone, message },
        };
        if succeeded {
            self.last_checked = Some(std::time::SystemTime::now());
        }
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

/// Reduce every reconciliation message synchronously.
pub(crate) fn update(
    state: &mut State,
    message: Message,
    context: Context<'_>,
) -> Outcome<Effect, Request> {
    match message {
        Message::Action(action) => update_action(state, action, context),
        Message::Load(zone) => load(state, zone),
        Message::ApplyPermanent => begin_mutation(
            context.mutation_pending,
            Mutation::ApplyPermanent,
            Effect::ApplyPermanent,
        ),
        Message::PersistRuntime => begin_mutation(
            context.mutation_pending,
            Mutation::PersistRuntime,
            Effect::PersistRuntime,
        ),
        Message::ConfigurationEvent(result) => match result {
            Ok(event) => {
                let blocked = context.mutation_pending || state.is_loading();
                match state.handle_configuration_event(blocked) {
                    RefreshRequest::Start => Outcome::request(Request::ConfigurationRefresh(event)),
                    RefreshRequest::Coalesced => Outcome::default(),
                }
            }
            Err(error) => {
                state.watcher_failed(error);
                Outcome::default()
            }
        },
        Message::LoadCompleted {
            zone,
            generation,
            result,
        } => {
            let current = context.selected_zone == Some(zone.as_str())
                && context.ready_zone == Some(zone.as_str());
            if !current || !state.complete_load(zone, generation, *result) {
                return Outcome::default();
            }
            finish_refresh(state)
        }
        Message::PermanentApplied(result) => {
            let succeeded = result.is_ok();
            let mut requests = vec![Request::FinishMutation(result)];
            if succeeded {
                requests.insert(0, Request::ClearRuntimeDirty);
            }
            requests.push(Request::RefreshFirewalldStatus);
            if succeeded {
                requests.push(Request::RefreshZones);
            }
            Outcome {
                effects: Vec::new(),
                requests,
            }
        }
        Message::RuntimePersisted(result) => {
            let succeeded = result.is_ok();
            let mut requests = vec![Request::FinishMutation(result)];
            if succeeded {
                requests.extend([
                    Request::RefreshFirewalldStatus,
                    Request::RefreshZones,
                    Request::RefreshIpSets,
                    Request::RefreshCatalogs,
                ]);
            }
            Outcome {
                effects: Vec::new(),
                requests,
            }
        }
    }
}

fn update_action(
    state: &mut State,
    action: ReconciliationAction,
    context: Context<'_>,
) -> Outcome<Effect, Request> {
    match action {
        ReconciliationAction::Review => Outcome::request(Request::OpenReview),
        ReconciliationAction::Refresh => {
            let Some(zone) = context.selected_zone else {
                return Outcome::default();
            };
            if !context.firewalld_active {
                state.set_unavailable(Some(zone.to_string()));
                return Outcome::default();
            }
            load(state, zone.to_string())
        }
        ReconciliationAction::ApplyPermanentToRuntime => {
            Outcome::request(Request::ConfirmApplyPermanent)
        }
        ReconciliationAction::SaveRuntimeAsPermanent => {
            Outcome::request(Request::ConfirmPersistRuntime)
        }
    }
}

fn load(state: &mut State, zone: String) -> Outcome<Effect, Request> {
    let generation = state.begin_load(zone.clone());
    Outcome::effect(Effect::Load { zone, generation })
}

fn begin_mutation(pending: bool, mutation: Mutation, effect: Effect) -> Outcome<Effect, Request> {
    if pending {
        Outcome::default()
    } else {
        Outcome {
            effects: vec![effect],
            requests: vec![Request::BeginMutation(mutation)],
        }
    }
}

/// Finish coordinated refresh work and schedule one coalesced follow-up.
pub(crate) fn finish_refresh(state: &mut State) -> Outcome<Effect, Request> {
    if state.refresh_finished() {
        Outcome::request(Request::ConfigurationRefresh(ConfigurationEvent::Reloaded))
    } else {
        Outcome::default()
    }
}

/// Run one reconciliation-owned effect.
pub(crate) fn effects(effect: Effect) -> Task<Message> {
    match effect {
        Effect::Load { zone, generation } => {
            let requested = zone.clone();
            Task::perform(load_reconciliation(zone), move |result| {
                Message::LoadCompleted {
                    zone: requested.clone(),
                    generation,
                    result: Box::new(result.map_err(|error| error.to_string())),
                }
            })
        }
        Effect::ApplyPermanent => Task::perform(apply_permanent(), Message::PermanentApplied),
        Effect::PersistRuntime => Task::perform(persist_runtime(), Message::RuntimePersisted),
    }
}

async fn load_reconciliation(zone: String) -> Result<ZoneReconciliationData, BrokerError> {
    let broker = FwdBroker::get().await?;
    broker.reconcile_zone(&zone).await
}

async fn apply_permanent() -> Result<(), BrokerError> {
    let broker = FwdBroker::get().await?;
    broker.apply_permanent_configuration().await
}

async fn persist_runtime() -> Result<(), BrokerError> {
    let broker = FwdBroker::get().await?;
    broker.persist_runtime_configuration().await
}

fn configuration_event_messages(selected_zone: Option<String>) -> BoxStream<'static, Message> {
    Box::pin(async_stream::stream! {
        let broker = match FwdBroker::get().await {
            Ok(broker) => broker,
            Err(error) => {
                yield Message::ConfigurationEvent(Err(error.to_string()));
                return;
            }
        };
        let mut events = broker.configuration_events(selected_zone);
        while let Some(event) = events.next().await {
            let failed = event.is_err();
            yield Message::ConfigurationEvent(event.map_err(|error| error.to_string()));
            if failed {
                return;
            }
        }
    })
}

/// Build the selected-zone configuration event stream for a keyed subscription.
pub(crate) fn configuration_event_subscription(
    selected_zone: &Option<String>,
) -> BoxStream<'static, Message> {
    configuration_event_messages(selected_zone.clone())
}

#[cfg(test)]
mod tests {
    use crate::core::{
        ComparisonCompleteness, RefreshRequest, ZoneReconciliationData, ZoneReconciliationState,
        reconciliation::{ZoneReconciliation, ZoneSettingsSnapshot},
    };

    use super::{Context, Message, ReconciliationAction, Request, State, update};

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

    fn context() -> Context<'static> {
        Context {
            selected_zone: Some("public"),
            ready_zone: Some("public"),
            firewalld_active: true,
            mutation_pending: false,
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

    #[test]
    fn global_directions_request_separate_confirmations() {
        let mut controller = State::default();

        let apply = update(
            &mut controller,
            Message::Action(ReconciliationAction::ApplyPermanentToRuntime),
            context(),
        );
        let persist = update(
            &mut controller,
            Message::Action(ReconciliationAction::SaveRuntimeAsPermanent),
            context(),
        );

        assert!(matches!(
            apply.requests.as_slice(),
            [Request::ConfirmApplyPermanent]
        ));
        assert!(matches!(
            persist.requests.as_slice(),
            [Request::ConfirmPersistRuntime]
        ));
    }

    #[test]
    fn permanent_apply_success_clears_dirty_before_finishing_and_refreshing() {
        let mut controller = State::default();

        let outcome = update(
            &mut controller,
            Message::PermanentApplied(Ok(())),
            context(),
        );

        assert!(matches!(
            outcome.requests.as_slice(),
            [
                Request::ClearRuntimeDirty,
                Request::FinishMutation(Ok(())),
                Request::RefreshFirewalldStatus,
                Request::RefreshZones
            ]
        ));
    }
}
