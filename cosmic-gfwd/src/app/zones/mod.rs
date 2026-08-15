//! Zone and firewalld feature state, reduction, effects, and presentation.

use std::collections::HashSet;

use crate::app::ContextPage;
use crate::app::dialogs::{DialogKind, PortKind};
use crate::app::outcome::Outcome;
use crate::app::reconciliation::ReconciliationAction;
use crate::core::{BrokerError, FirewalldStatus};
use crate::models::{ZoneDetails, ZoneTarget};

mod effects;
pub(crate) use effects::effects;
mod view;

pub(crate) use view::{ZoneViewAction, ZoneViewState, view_zone_content};

/// Zone-owned detail projection and ordinary daemon status.
#[derive(Debug)]
pub(crate) struct State {
    detail: ZoneViewState,
    firewalld_status: FirewalldStatus,
}

impl Default for State {
    fn default() -> Self {
        Self {
            detail: ZoneViewState::Empty,
            firewalld_status: FirewalldStatus::Loading,
        }
    }
}

impl State {
    /// Return the selected-zone detail projection.
    pub(crate) fn detail(&self) -> &ZoneViewState {
        &self.detail
    }

    /// Return ready details, when the selected zone has loaded.
    pub(crate) fn ready_detail(&self) -> Option<&ZoneDetails> {
        match &self.detail {
            ZoneViewState::Ready(details) => Some(details),
            _ => None,
        }
    }

    /// Return the zone represented by the ready detail projection.
    pub(crate) fn current_zone_name(&self) -> Option<&str> {
        self.ready_detail().map(|details| details.name.as_str())
    }

    /// Return ordinary firewalld daemon status.
    pub(crate) fn firewalld_status(&self) -> &FirewalldStatus {
        &self.firewalld_status
    }
}

/// Immutable root projections needed while reducing one zone message.
pub(crate) struct Context<'a> {
    pub(crate) mutation_pending: bool,
    pub(crate) selected_zone: Option<&'a str>,
    pub(crate) reconciliation_refreshing: bool,
    pub(crate) open_dialog: Option<DialogKind>,
}

/// Zone and firewalld messages delivered to the feature reducer.
#[derive(Clone, Debug)]
pub(crate) enum Message {
    View(ZoneViewAction),
    LoadList,
    LoadDetails(String),
    LoadDefault,
    LoadActive,
    LoadStatus,
    ClearSelection,
    ShowListError(String),
    SetDefault(String),
    Create {
        name: String,
        description: String,
        target: ZoneTarget,
    },
    Rename {
        old_name: String,
        new_name: String,
    },
    Delete(String),
    AddService {
        zone: String,
        service: String,
    },
    AddPort {
        zone: String,
        port: String,
        protocol: String,
    },
    AddSourcePort {
        zone: String,
        port: String,
        protocol: String,
    },
    AddForwardPort {
        zone: String,
        port: String,
        protocol: String,
        to_port: String,
        to_addr: String,
    },
    AddInterface {
        zone: String,
        interface: String,
    },
    AddSource {
        zone: String,
        source: String,
    },
    AddIcmp {
        zone: String,
        icmp: String,
    },
    AddRichRule {
        zone: String,
        rule: String,
    },
    Remove(ZoneViewAction),
    SetMasquerade(bool),
    SetIcmpBlockInversion(bool),
    ControlFirewalld(bool),
    ConfirmDelete(String),
    ListLoaded(Result<Vec<String>, BrokerError>),
    DetailsLoaded {
        zone_name: String,
        result: Box<Result<ZoneDetails, BrokerError>>,
    },
    DefaultLoaded(Result<String, BrokerError>),
    ActiveLoaded(Result<HashSet<String>, BrokerError>),
    DefaultSet(Result<(), BrokerError>),
    Created {
        zone_name: String,
        result: Result<(), BrokerError>,
    },
    Renamed {
        old_name: String,
        new_name: String,
        result: Result<(), BrokerError>,
    },
    Deleted {
        zone_name: String,
        result: Result<(), BrokerError>,
    },
    ItemAdded {
        zone_name: String,
        result: Result<(), BrokerError>,
    },
    ItemRemoved {
        zone_name: String,
        result: Result<(), BrokerError>,
    },
    FirewalldStatusLoaded(Result<FirewalldStatus, BrokerError>),
    DaemonControlFinished(Result<(), BrokerError>),
}

/// Broker-backed asynchronous work owned by the zone feature.
#[derive(Clone, Debug)]
pub(crate) enum Effect {
    LoadZones,
    LoadDetails(String),
    LoadDefault,
    LoadActive,
    LoadStatus,
    SetDefault(String),
    CreateZone {
        name: String,
        description: String,
        target: ZoneTarget,
    },
    RenameZone {
        old_name: String,
        new_name: String,
    },
    DeleteZone(String),
    AddService {
        zone: String,
        service: String,
    },
    AddPort {
        zone: String,
        port: String,
        protocol: String,
    },
    AddSourcePort {
        zone: String,
        port: String,
        protocol: String,
    },
    AddForwardPort {
        zone: String,
        port: String,
        protocol: String,
        to_port: String,
        to_addr: String,
    },
    AddInterface {
        zone: String,
        interface: String,
    },
    AddSource {
        zone: String,
        source: String,
    },
    AddIcmp {
        zone: String,
        icmp: String,
    },
    AddRichRule {
        zone: String,
        rule: String,
    },
    RemoveService {
        zone: String,
        service: String,
    },
    RemoveInterface {
        zone: String,
        interface: String,
    },
    RemoveSource {
        zone: String,
        source: String,
    },
    RemovePort {
        zone: String,
        port: String,
        protocol: String,
    },
    RemoveForwardPort {
        zone: String,
        port: String,
        protocol: String,
        to_port: String,
        to_addr: String,
    },
    RemoveSourcePort {
        zone: String,
        port: String,
        protocol: String,
    },
    RemoveIcmp {
        zone: String,
        icmp: String,
    },
    RemoveRichRule {
        zone: String,
        rule: String,
    },
    SetMasquerade {
        zone: String,
        enabled: bool,
    },
    SetIcmpBlockInversion {
        zone: String,
        enabled: bool,
    },
    ControlFirewalld(bool),
}

/// Neutral mutation kinds localized by the root.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Mutation {
    CreateZone,
    RenameZone,
    DeleteZone,
    AddService,
    AddPort,
    AddSourcePort,
    AddForwardPort,
    AddInterface,
    AddSource,
    AddIcmp,
    AddRichRule,
    RemoveItem,
    SetMasquerade,
    SetIcmpBlockInversion,
    SetDefaultZone,
    StartFirewalld,
    StopFirewalld,
}

/// Root-owned coordination emitted by zone reduction.
#[derive(Debug)]
pub(crate) enum Request {
    NavigationLoading,
    NavigationZonesLoaded(Result<Vec<String>, String>),
    NavigationDefaultLoaded(Result<String, String>),
    NavigationActiveLoaded(Result<HashSet<String>, String>),
    OpenContextPage(ContextPage),
    SetPortKind(PortKind),
    ResetDialog(DialogKind),
    ReconciliationSelectionChanged(Option<String>),
    LoadReconciliation(String),
    ReconciliationUnavailable(Option<String>),
    PreserveZoneRename { old_name: String, new_name: String },
    ReconciliationAction(ReconciliationAction),
    FinishConfigurationRefresh,
    ConfirmDeleteZone(String),
    ConfirmStopFirewalld,
    BeginMutation(Mutation),
    FinishMutation(Result<(), BrokerError>),
    MarkRuntimeDirty,
    CloseDrawer,
    RefreshZones,
    RefreshDefault,
    RefreshStatus,
    RefreshCurrentZone(String),
}

/// Reduce a zone message and return root requests and broker effects in causal order.
pub(crate) fn update(
    state: &mut State,
    message: Message,
    context: Context<'_>,
) -> Outcome<Effect, Request> {
    match message {
        Message::View(action) => match action {
            ZoneViewAction::SetMasquerade(enabled) => {
                update(state, Message::SetMasquerade(enabled), context)
            }
            ZoneViewAction::SetIcmpBlockInversion(enabled) => {
                update(state, Message::SetIcmpBlockInversion(enabled), context)
            }
            action @ (ZoneViewAction::RemoveService(_)
            | ZoneViewAction::RemoveInterface(_)
            | ZoneViewAction::RemoveSource(_)
            | ZoneViewAction::RemovePort { .. }
            | ZoneViewAction::RemoveForwardPort { .. }
            | ZoneViewAction::RemoveSourcePort { .. }
            | ZoneViewAction::RemoveIcmpBlock(_)
            | ZoneViewAction::RemoveRichRule(_)) => update(state, Message::Remove(action), context),
            ZoneViewAction::StartFirewalld => {
                update(state, Message::ControlFirewalld(true), context)
            }
            action => update_view(action, context),
        },
        Message::LoadList => Outcome {
            effects: vec![Effect::LoadZones],
            requests: vec![Request::NavigationLoading],
        },
        Message::LoadDetails(zone) => {
            state.detail = ZoneViewState::Loading { zone: zone.clone() };
            Outcome {
                effects: vec![Effect::LoadDetails(zone.clone())],
                requests: vec![Request::ReconciliationSelectionChanged(Some(zone))],
            }
        }
        Message::LoadDefault => Outcome::effect(Effect::LoadDefault),
        Message::LoadActive => Outcome::effect(Effect::LoadActive),
        Message::LoadStatus => {
            state.firewalld_status = FirewalldStatus::Loading;
            Outcome::effect(Effect::LoadStatus)
        }
        Message::ClearSelection => {
            state.detail = ZoneViewState::Empty;
            Outcome::default()
        }
        Message::ShowListError(message) => {
            state.detail = ZoneViewState::Error {
                zone: "zones".to_string(),
                message,
            };
            Outcome::default()
        }
        Message::SetDefault(zone) => begin_effect(
            context.mutation_pending,
            Mutation::SetDefaultZone,
            Effect::SetDefault(zone),
        ),
        Message::Create {
            name,
            description,
            target,
        } => begin_effect(
            context.mutation_pending,
            Mutation::CreateZone,
            Effect::CreateZone {
                name,
                description,
                target,
            },
        ),
        Message::Rename { old_name, new_name } => begin_effect(
            context.mutation_pending,
            Mutation::RenameZone,
            Effect::RenameZone { old_name, new_name },
        ),
        Message::Delete(zone) => begin_effect(
            context.mutation_pending,
            Mutation::DeleteZone,
            Effect::DeleteZone(zone),
        ),
        Message::AddService { zone, service } => begin_effect(
            context.mutation_pending,
            Mutation::AddService,
            Effect::AddService { zone, service },
        ),
        Message::AddPort {
            zone,
            port,
            protocol,
        } => begin_effect(
            context.mutation_pending,
            Mutation::AddPort,
            Effect::AddPort {
                zone,
                port,
                protocol,
            },
        ),
        Message::AddSourcePort {
            zone,
            port,
            protocol,
        } => begin_effect(
            context.mutation_pending,
            Mutation::AddSourcePort,
            Effect::AddSourcePort {
                zone,
                port,
                protocol,
            },
        ),
        Message::AddForwardPort {
            zone,
            port,
            protocol,
            to_port,
            to_addr,
        } => begin_effect(
            context.mutation_pending,
            Mutation::AddForwardPort,
            Effect::AddForwardPort {
                zone,
                port,
                protocol,
                to_port,
                to_addr,
            },
        ),
        Message::AddInterface { zone, interface } => begin_effect(
            context.mutation_pending,
            Mutation::AddInterface,
            Effect::AddInterface { zone, interface },
        ),
        Message::AddSource { zone, source } => begin_effect(
            context.mutation_pending,
            Mutation::AddSource,
            Effect::AddSource { zone, source },
        ),
        Message::AddIcmp { zone, icmp } => begin_effect(
            context.mutation_pending,
            Mutation::AddIcmp,
            Effect::AddIcmp { zone, icmp },
        ),
        Message::AddRichRule { zone, rule } => begin_effect(
            context.mutation_pending,
            Mutation::AddRichRule,
            Effect::AddRichRule { zone, rule },
        ),
        Message::Remove(action) => remove_item(state, action, context.mutation_pending),
        Message::SetMasquerade(enabled) => set_masquerade(state, enabled, context.mutation_pending),
        Message::SetIcmpBlockInversion(enabled) => {
            set_icmp_inversion(state, enabled, context.mutation_pending)
        }
        Message::ControlFirewalld(start) => {
            control_firewalld(state, start, context.mutation_pending)
        }
        Message::ConfirmDelete(zone) => Outcome::request(Request::ConfirmDeleteZone(zone)),
        Message::ListLoaded(result) => Outcome::request(Request::NavigationZonesLoaded(
            result.map_err(|error| error.to_string()),
        )),
        Message::DefaultLoaded(result) => Outcome::request(Request::NavigationDefaultLoaded(
            result.map_err(|error| error.to_string()),
        )),
        Message::ActiveLoaded(result) => Outcome::request(Request::NavigationActiveLoaded(
            result.map_err(|error| error.to_string()),
        )),
        Message::DetailsLoaded { zone_name, result } => {
            finish_details(state, zone_name, *result, context)
        }
        Message::FirewalldStatusLoaded(result) => finish_status(state, result, context),
        Message::DefaultSet(result) => finish_default(result),
        Message::Created { zone_name, result } => finish_create(zone_name, result),
        Message::Renamed {
            old_name,
            new_name,
            result,
        } => finish_rename(old_name, new_name, result),
        Message::Deleted { zone_name, result } => {
            finish_delete(state, zone_name, result, context.selected_zone)
        }
        Message::ItemAdded { zone_name, result } | Message::ItemRemoved { zone_name, result } => {
            finish_item_change(zone_name, result, context)
        }
        Message::DaemonControlFinished(result) => finish_daemon(result),
    }
}

fn update_view(action: ZoneViewAction, context: Context<'_>) -> Outcome<Effect, Request> {
    if context.mutation_pending {
        return Outcome::default();
    }

    match action {
        ZoneViewAction::Reconciliation(action) => {
            Outcome::request(Request::ReconciliationAction(action))
        }
        ZoneViewAction::AddService => {
            Outcome::request(Request::OpenContextPage(ContextPage::AddService))
        }
        ZoneViewAction::AddInterface => {
            Outcome::request(Request::OpenContextPage(ContextPage::AddInterface))
        }
        ZoneViewAction::AddPort { kind } => Outcome {
            effects: Vec::new(),
            requests: vec![
                Request::OpenContextPage(ContextPage::AddPort),
                Request::SetPortKind(kind),
            ],
        },
        ZoneViewAction::AddSource => {
            Outcome::request(Request::OpenContextPage(ContextPage::AddSource))
        }
        ZoneViewAction::AddIcmpBlock => {
            Outcome::request(Request::OpenContextPage(ContextPage::AddIcmp))
        }
        ZoneViewAction::AddRichRule => {
            Outcome::request(Request::OpenContextPage(ContextPage::AddRichRule))
        }
        ZoneViewAction::StartFirewalld => unreachable!("view commands are normalized first"),
        ZoneViewAction::StopFirewalld => Outcome::request(Request::ConfirmStopFirewalld),
        ZoneViewAction::SetMasquerade(_)
        | ZoneViewAction::SetIcmpBlockInversion(_)
        | ZoneViewAction::RemoveService(_)
        | ZoneViewAction::RemoveInterface(_)
        | ZoneViewAction::RemoveSource(_)
        | ZoneViewAction::RemovePort { .. }
        | ZoneViewAction::RemoveForwardPort { .. }
        | ZoneViewAction::RemoveSourcePort { .. }
        | ZoneViewAction::RemoveIcmpBlock(_)
        | ZoneViewAction::RemoveRichRule(_) => unreachable!("view commands are normalized first"),
    }
}

fn begin_effect(
    mutation_pending: bool,
    mutation: Mutation,
    effect: Effect,
) -> Outcome<Effect, Request> {
    if mutation_pending {
        Outcome::default()
    } else {
        Outcome {
            effects: vec![effect],
            requests: vec![Request::BeginMutation(mutation)],
        }
    }
}

fn set_masquerade(
    state: &State,
    enabled: bool,
    mutation_pending: bool,
) -> Outcome<Effect, Request> {
    let Some(zone) = state.current_zone_name() else {
        return Outcome::default();
    };
    begin_effect(
        mutation_pending,
        Mutation::SetMasquerade,
        Effect::SetMasquerade {
            zone: zone.to_string(),
            enabled,
        },
    )
}

fn set_icmp_inversion(
    state: &State,
    enabled: bool,
    mutation_pending: bool,
) -> Outcome<Effect, Request> {
    let Some(zone) = state.current_zone_name() else {
        return Outcome::default();
    };
    begin_effect(
        mutation_pending,
        Mutation::SetIcmpBlockInversion,
        Effect::SetIcmpBlockInversion {
            zone: zone.to_string(),
            enabled,
        },
    )
}

fn control_firewalld(
    state: &mut State,
    start: bool,
    mutation_pending: bool,
) -> Outcome<Effect, Request> {
    if mutation_pending {
        return Outcome::default();
    }
    state.firewalld_status = FirewalldStatus::Loading;
    let mutation = if start {
        Mutation::StartFirewalld
    } else {
        Mutation::StopFirewalld
    };
    begin_effect(false, mutation, Effect::ControlFirewalld(start))
}

fn remove_item(
    state: &State,
    action: ZoneViewAction,
    mutation_pending: bool,
) -> Outcome<Effect, Request> {
    let Some(zone) = state.current_zone_name().map(str::to_string) else {
        return Outcome::default();
    };
    let effect = match action {
        ZoneViewAction::RemoveService(service) => Effect::RemoveService { zone, service },
        ZoneViewAction::RemoveInterface(interface) => Effect::RemoveInterface { zone, interface },
        ZoneViewAction::RemoveSource(source) => Effect::RemoveSource { zone, source },
        ZoneViewAction::RemovePort { port, protocol } => Effect::RemovePort {
            zone,
            port,
            protocol,
        },
        ZoneViewAction::RemoveForwardPort {
            port,
            protocol,
            to_port,
            to_addr,
        } => Effect::RemoveForwardPort {
            zone,
            port,
            protocol,
            to_port,
            to_addr,
        },
        ZoneViewAction::RemoveSourcePort { port, protocol } => Effect::RemoveSourcePort {
            zone,
            port,
            protocol,
        },
        ZoneViewAction::RemoveIcmpBlock(icmp) => Effect::RemoveIcmp { zone, icmp },
        ZoneViewAction::RemoveRichRule(rule) => Effect::RemoveRichRule { zone, rule },
        _ => return Outcome::default(),
    };
    begin_effect(mutation_pending, Mutation::RemoveItem, effect)
}

fn finish_details(
    state: &mut State,
    zone: String,
    result: Result<ZoneDetails, BrokerError>,
    context: Context<'_>,
) -> Outcome<Effect, Request> {
    if context.selected_zone != Some(zone.as_str()) {
        return Outcome::default();
    }

    match result {
        Ok(details) => {
            state.detail = ZoneViewState::Ready(Box::new(details));
            if state.firewalld_status == FirewalldStatus::Active {
                Outcome::request(Request::LoadReconciliation(zone))
            } else {
                Outcome {
                    effects: Vec::new(),
                    requests: vec![
                        Request::ReconciliationUnavailable(Some(zone)),
                        Request::FinishConfigurationRefresh,
                    ],
                }
            }
        }
        Err(error) => {
            state.detail = ZoneViewState::Error {
                zone: zone.clone(),
                message: error.to_string(),
            };
            Outcome {
                effects: Vec::new(),
                requests: vec![
                    Request::ReconciliationUnavailable(Some(zone)),
                    Request::FinishConfigurationRefresh,
                ],
            }
        }
    }
}

fn finish_status(
    state: &mut State,
    result: Result<FirewalldStatus, BrokerError>,
    context: Context<'_>,
) -> Outcome<Effect, Request> {
    state.firewalld_status = match result {
        Ok(status) => status,
        Err(error) => FirewalldStatus::Error(error.to_string()),
    };

    if state.firewalld_status == FirewalldStatus::Active
        && !context.reconciliation_refreshing
        && let Some(zone) = state.current_zone_name()
    {
        return Outcome::request(Request::LoadReconciliation(zone.to_string()));
    }
    Outcome::request(Request::ReconciliationUnavailable(
        state.current_zone_name().map(str::to_string),
    ))
}

fn finish_default(result: Result<(), BrokerError>) -> Outcome<Effect, Request> {
    match result {
        Ok(()) => Outcome {
            effects: Vec::new(),
            requests: vec![Request::FinishMutation(Ok(())), Request::RefreshDefault],
        },
        Err(error) => Outcome::request(Request::FinishMutation(Err(error))),
    }
}

fn finish_create(_zone: String, result: Result<(), BrokerError>) -> Outcome<Effect, Request> {
    match result {
        Ok(()) => Outcome {
            effects: Vec::new(),
            requests: vec![
                Request::MarkRuntimeDirty,
                Request::ResetDialog(DialogKind::Zone),
                Request::CloseDrawer,
                Request::FinishMutation(Ok(())),
                Request::RefreshZones,
            ],
        },
        Err(error) => Outcome::request(Request::FinishMutation(Err(error))),
    }
}

fn finish_rename(
    old_name: String,
    new_name: String,
    result: Result<(), BrokerError>,
) -> Outcome<Effect, Request> {
    match result {
        Ok(()) => Outcome {
            effects: Vec::new(),
            requests: vec![
                Request::PreserveZoneRename { old_name, new_name },
                Request::MarkRuntimeDirty,
                Request::ResetDialog(DialogKind::Zone),
                Request::CloseDrawer,
                Request::FinishMutation(Ok(())),
                Request::RefreshZones,
            ],
        },
        Err(error) => Outcome::request(Request::FinishMutation(Err(error))),
    }
}

fn finish_delete(
    state: &mut State,
    zone: String,
    result: Result<(), BrokerError>,
    selected_zone: Option<&str>,
) -> Outcome<Effect, Request> {
    match result {
        Ok(()) => {
            let mut requests = Vec::new();
            if selected_zone == Some(zone.as_str()) {
                state.detail = ZoneViewState::Empty;
                requests.push(Request::ReconciliationUnavailable(None));
            }
            requests.extend([
                Request::MarkRuntimeDirty,
                Request::FinishMutation(Ok(())),
                Request::RefreshZones,
            ]);
            Outcome {
                effects: Vec::new(),
                requests,
            }
        }
        Err(error) => Outcome::request(Request::FinishMutation(Err(error))),
    }
}

fn finish_item_change(
    zone: String,
    result: Result<(), BrokerError>,
    context: Context<'_>,
) -> Outcome<Effect, Request> {
    match result {
        Ok(()) => {
            let mut requests = vec![Request::MarkRuntimeDirty];
            if let Some(dialog) = context.open_dialog {
                requests.push(Request::ResetDialog(dialog));
                requests.push(Request::CloseDrawer);
            }
            requests.push(Request::FinishMutation(Ok(())));
            if context.selected_zone == Some(zone.as_str()) {
                requests.push(Request::RefreshCurrentZone(zone));
            }
            Outcome {
                effects: Vec::new(),
                requests,
            }
        }
        Err(error) => Outcome::request(Request::FinishMutation(Err(error))),
    }
}

fn finish_daemon(result: Result<(), BrokerError>) -> Outcome<Effect, Request> {
    match result {
        Ok(()) => Outcome {
            effects: Vec::new(),
            requests: vec![Request::FinishMutation(Ok(())), Request::RefreshStatus],
        },
        Err(error) => Outcome::request(Request::FinishMutation(Err(error))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn context(selected_zone: Option<&str>) -> Context<'_> {
        Context {
            mutation_pending: false,
            selected_zone,
            reconciliation_refreshing: false,
            open_dialog: None,
        }
    }

    fn details(name: &str) -> ZoneDetails {
        ZoneDetails {
            name: name.into(),
            description: String::new(),
            target: ZoneTarget::Default,
            masquerade: false,
            icmp_block_inversion: false,
            services: Vec::new(),
            ports: Vec::new(),
            forward_ports: Vec::new(),
            interfaces: Vec::new(),
            sources: Vec::new(),
            icmp_blocks: Vec::new(),
            rich_rules: Vec::new(),
            protocols: Vec::new(),
            source_ports: Vec::new(),
        }
    }

    #[test]
    fn selected_zone_load_changes_reconciliation_identity_before_effect() {
        let mut state = State::default();
        let outcome = update(
            &mut state,
            Message::LoadDetails("public".into()),
            context(Some("public")),
        );

        assert!(matches!(
            outcome.requests.as_slice(),
            [Request::ReconciliationSelectionChanged(Some(zone))] if zone == "public"
        ));
        assert!(matches!(
            outcome.effects.as_slice(),
            [Effect::LoadDetails(zone)] if zone == "public"
        ));
        assert!(matches!(
            state.detail(),
            ZoneViewState::Loading { zone } if zone == "public"
        ));
    }

    #[test]
    fn mutation_commands_are_gated_by_the_global_slot() {
        let mut state = State::default();
        let outcome = update(
            &mut state,
            Message::SetDefault("public".into()),
            Context {
                mutation_pending: true,
                ..context(None)
            },
        );

        assert!(outcome.effects.is_empty());
        assert!(outcome.requests.is_empty());
    }

    #[test]
    fn successful_create_emits_root_requests_in_causal_order() {
        let mut state = State::default();
        let outcome = update(
            &mut state,
            Message::Created {
                zone_name: "work".into(),
                result: Ok(()),
            },
            context(None),
        );

        assert!(matches!(
            outcome.requests.as_slice(),
            [
                Request::MarkRuntimeDirty,
                Request::ResetDialog(DialogKind::Zone),
                Request::CloseDrawer,
                Request::FinishMutation(Ok(())),
                Request::RefreshZones,
            ]
        ));
    }

    #[test]
    fn stale_detail_completion_is_ignored() {
        let mut state = State::default();
        let outcome = update(
            &mut state,
            Message::DetailsLoaded {
                zone_name: "public".into(),
                result: Box::new(Ok(ZoneDetails {
                    name: "public".into(),
                    description: String::new(),
                    target: ZoneTarget::Default,
                    masquerade: false,
                    icmp_block_inversion: false,
                    services: Vec::new(),
                    ports: Vec::new(),
                    forward_ports: Vec::new(),
                    interfaces: Vec::new(),
                    sources: Vec::new(),
                    icmp_blocks: Vec::new(),
                    rich_rules: Vec::new(),
                    protocols: Vec::new(),
                    source_ports: Vec::new(),
                })),
            },
            context(Some("work")),
        );

        assert!(outcome.effects.is_empty());
        assert!(outcome.requests.is_empty());
        assert!(matches!(state.detail(), ZoneViewState::Empty));
    }

    #[test]
    fn successful_item_change_resets_dialog_before_finishing_and_reloading() {
        let mut state = State::default();
        let outcome = update(
            &mut state,
            Message::ItemAdded {
                zone_name: "public".into(),
                result: Ok(()),
            },
            Context {
                open_dialog: Some(DialogKind::Service),
                ..context(Some("public"))
            },
        );

        assert!(matches!(
            outcome.requests.as_slice(),
            [
                Request::MarkRuntimeDirty,
                Request::ResetDialog(DialogKind::Service),
                Request::CloseDrawer,
                Request::FinishMutation(Ok(())),
                Request::RefreshCurrentZone(zone),
            ] if zone == "public"
        ));
    }

    #[test]
    fn view_add_actions_emit_context_pages_and_dialog_configuration() {
        let mut state = State::default();
        let service = update(
            &mut state,
            Message::View(ZoneViewAction::AddService),
            context(None),
        );
        assert!(matches!(
            service.requests.as_slice(),
            [Request::OpenContextPage(ContextPage::AddService)]
        ));

        let port = update(
            &mut state,
            Message::View(ZoneViewAction::AddPort {
                kind: PortKind::Source,
            }),
            context(None),
        );
        assert!(matches!(
            port.requests.as_slice(),
            [
                Request::OpenContextPage(ContextPage::AddPort),
                Request::SetPortKind(PortKind::Source),
            ]
        ));
    }

    #[test]
    fn view_mutations_require_ready_details_and_respect_global_gating() {
        let mut state = State::default();
        let missing = update(
            &mut state,
            Message::View(ZoneViewAction::RemoveService("ssh".into())),
            context(Some("public")),
        );
        assert!(missing.effects.is_empty());
        assert!(missing.requests.is_empty());

        state.detail = ZoneViewState::Ready(Box::new(details("public")));
        let blocked = update(
            &mut state,
            Message::View(ZoneViewAction::SetMasquerade(true)),
            Context {
                mutation_pending: true,
                ..context(Some("public"))
            },
        );
        assert!(blocked.effects.is_empty());
        assert!(blocked.requests.is_empty());

        let removal = update(
            &mut state,
            Message::View(ZoneViewAction::RemoveService("ssh".into())),
            context(Some("public")),
        );
        assert!(matches!(
            removal.requests.as_slice(),
            [Request::BeginMutation(Mutation::RemoveItem)]
        ));
        assert!(matches!(
            removal.effects.as_slice(),
            [Effect::RemoveService { zone, service }]
                if zone == "public" && service == "ssh"
        ));
    }

    #[test]
    fn stop_is_confirmed_before_daemon_control_begins() {
        let mut state = State::default();
        let request = update(
            &mut state,
            Message::View(ZoneViewAction::StopFirewalld),
            context(None),
        );
        assert!(matches!(
            request.requests.as_slice(),
            [Request::ConfirmStopFirewalld]
        ));
        assert!(request.effects.is_empty());

        let confirmed = update(&mut state, Message::ControlFirewalld(false), context(None));
        assert_eq!(state.firewalld_status(), &FirewalldStatus::Loading);
        assert!(matches!(
            confirmed.requests.as_slice(),
            [Request::BeginMutation(Mutation::StopFirewalld)]
        ));
        assert!(matches!(
            confirmed.effects.as_slice(),
            [Effect::ControlFirewalld(false)]
        ));
    }

    #[test]
    fn inactive_detail_completion_marks_reconciliation_unavailable_then_finishes_refresh() {
        let mut state = State {
            detail: ZoneViewState::Loading {
                zone: "public".into(),
            },
            firewalld_status: FirewalldStatus::Inactive,
        };
        let outcome = update(
            &mut state,
            Message::DetailsLoaded {
                zone_name: "public".into(),
                result: Box::new(Ok(details("public"))),
            },
            context(Some("public")),
        );

        assert!(matches!(
            outcome.requests.as_slice(),
            [
                Request::ReconciliationUnavailable(Some(zone)),
                Request::FinishConfigurationRefresh,
            ] if zone == "public"
        ));
        assert_eq!(state.current_zone_name(), Some("public"));
    }

    #[test]
    fn active_status_loads_ready_reconciliation_or_marks_it_unavailable_while_refreshing() {
        let mut state = State {
            detail: ZoneViewState::Ready(Box::new(details("public"))),
            firewalld_status: FirewalldStatus::Loading,
        };
        let load = update(
            &mut state,
            Message::FirewalldStatusLoaded(Ok(FirewalldStatus::Active)),
            context(Some("public")),
        );
        assert!(matches!(
            load.requests.as_slice(),
            [Request::LoadReconciliation(zone)] if zone == "public"
        ));

        let refreshing = update(
            &mut state,
            Message::FirewalldStatusLoaded(Ok(FirewalldStatus::Active)),
            Context {
                reconciliation_refreshing: true,
                ..context(Some("public"))
            },
        );
        assert!(matches!(
            refreshing.requests.as_slice(),
            [Request::ReconciliationUnavailable(Some(zone))] if zone == "public"
        ));
    }

    #[test]
    fn default_and_daemon_completions_finish_before_their_refreshes() {
        let mut state = State::default();
        let default = update(&mut state, Message::DefaultSet(Ok(())), context(None));
        assert!(matches!(
            default.requests.as_slice(),
            [Request::FinishMutation(Ok(())), Request::RefreshDefault]
        ));

        let daemon = update(
            &mut state,
            Message::DaemonControlFinished(Ok(())),
            context(None),
        );
        assert!(matches!(
            daemon.requests.as_slice(),
            [Request::FinishMutation(Ok(())), Request::RefreshStatus]
        ));
    }

    #[test]
    fn deleting_the_selected_zone_clears_details_before_dirty_finish_and_reload() {
        let mut state = State {
            detail: ZoneViewState::Ready(Box::new(details("public"))),
            firewalld_status: FirewalldStatus::Active,
        };
        let outcome = update(
            &mut state,
            Message::Deleted {
                zone_name: "public".into(),
                result: Ok(()),
            },
            context(Some("public")),
        );

        assert!(matches!(state.detail(), ZoneViewState::Empty));
        assert!(matches!(
            outcome.requests.as_slice(),
            [
                Request::ReconciliationUnavailable(None),
                Request::MarkRuntimeDirty,
                Request::FinishMutation(Ok(())),
                Request::RefreshZones,
            ]
        ));
    }

    #[test]
    fn non_current_item_success_finishes_without_reloading_details() {
        let mut state = State::default();
        let outcome = update(
            &mut state,
            Message::ItemRemoved {
                zone_name: "work".into(),
                result: Ok(()),
            },
            context(Some("public")),
        );

        assert!(matches!(
            outcome.requests.as_slice(),
            [Request::MarkRuntimeDirty, Request::FinishMutation(Ok(()))]
        ));
    }
}
