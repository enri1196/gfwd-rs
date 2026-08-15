// SPDX-License-Identifier: MIT

use crate::config::Config;
use crate::core::{BrokerError, ConfigurationEvent, FirewalldStatus};
use crate::fl;
use cosmic::cosmic_config::{self, CosmicConfigEntry};
use cosmic::iced::Subscription;
use cosmic::prelude::*;
use cosmic::widget::{self, Toast, ToastId, about::About, menu, nav_bar};
use dialogs::{DialogKind, DialogMessage, DialogState, localized_validation_error};
use navigation::{MenuAction as NavMenuAction, SidebarItem};
use std::collections::HashMap;
use zones::{ZoneViewAction, ZoneViewState};

const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");
const APP_ICON: &[u8] = include_bytes!("../../resources/icons/hicolor/scalable/apps/icon.svg");

mod catalogs;
pub(crate) mod dialogs;
mod ipsets;
mod navigation;
mod operations;
mod outcome;
pub(crate) mod reconciliation;
mod router;
mod view;
mod zones;

/// The application model stores app-specific state used to describe its interface and
/// drive its logic.
pub struct AppModel {
    /// Application state which is managed by the COSMIC runtime.
    core: cosmic::Core,
    /// Display a context drawer with the designated page if defined.
    context_page: ContextPage,
    /// The about page for this app.
    about: About,
    /// Contains items assigned to the nav bar panel.
    navigation: navigation::State,
    /// State for the current zone detail view.
    zones: zones::State,
    /// Selected-zone reconciliation lifecycle and refresh coordination.
    reconciliation: reconciliation::State,
    /// State for the IP set view.
    ipsets: ipsets::State,
    /// Stores form state for context drawer dialogs.
    dialogs: DialogState,
    /// Typed option catalogs used by dialog forms.
    catalogs: catalogs::State,
    /// Globally serialized mutations, confirmations, reload state, and notifications.
    operations: operations::State<Message, Confirmation>,
    /// Current activation state of the firewalld systemd unit.
    firewalld_status: FirewalldStatus,
    /// Key bindings for the application's menu bar.
    key_binds: HashMap<menu::KeyBind, MenuAction>,
    /// Configuration data that persists between application runs.
    config: Config,
}

/// Messages emitted by the application and its widgets.
#[derive(Debug, Clone)]
pub enum Message {
    Navigation(navigation::Message),
    Catalog(catalogs::Message),
    IpSet(ipsets::Message),
    Zone(zones::Message),
    Dialog(DialogMessage),
    Reconciliation(reconciliation::Message),

    DismissToast(ToastId),
    CancelConfirmation,
    ConfirmDestructive,
}

/// Destructive action presented in the application confirmation dialog.
#[derive(Debug, Clone)]
enum Confirmation {
    DeleteZone(String),
    DeleteIpSet(String),
    StopFirewalld,
    ApplyPermanentConfiguration,
    SaveRuntimeConfiguration,
}

/// Work that must be applied by the root before feature effects are scheduled.
enum RootRequest {
    DismissToast(ToastId),
}

/// Explicit root execution plan for one validated dialog request.
#[derive(Debug, Clone, Eq, PartialEq)]
enum DialogRoute {
    CreateZone {
        name: String,
        description: String,
        target: crate::models::ZoneTarget,
    },
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
    CreateIpSet {
        name: String,
        ipset_type: String,
        entries: Vec<String>,
    },
    CloseDrawer,
}

fn plan_dialog_request(request: dialogs::Request) -> DialogRoute {
    match request {
        dialogs::Request::Submit(submission) => match submission {
            dialogs::Submission::Zone {
                name,
                description,
                target,
            } => DialogRoute::CreateZone {
                name,
                description,
                target,
            },
            dialogs::Submission::Service { zone, service } => {
                DialogRoute::AddService { zone, service }
            }
            dialogs::Submission::Port {
                zone,
                port,
                protocol,
            } => DialogRoute::AddPort {
                zone,
                port,
                protocol,
            },
            dialogs::Submission::SourcePort {
                zone,
                port,
                protocol,
            } => DialogRoute::AddSourcePort {
                zone,
                port,
                protocol,
            },
            dialogs::Submission::ForwardPort {
                zone,
                port,
                protocol,
                to_port,
                to_addr,
            } => DialogRoute::AddForwardPort {
                zone,
                port,
                protocol,
                to_port,
                to_addr,
            },
            dialogs::Submission::Interface { zone, interface } => {
                DialogRoute::AddInterface { zone, interface }
            }
            dialogs::Submission::Source { zone, source } => DialogRoute::AddSource { zone, source },
            dialogs::Submission::Icmp { zone, icmp } => DialogRoute::AddIcmp { zone, icmp },
            dialogs::Submission::RichRule { zone, rule } => DialogRoute::AddRichRule { zone, rule },
            dialogs::Submission::IpSet {
                name,
                ipset_type,
                entries,
            } => DialogRoute::CreateIpSet {
                name,
                ipset_type,
                entries,
            },
        },
        dialogs::Request::CloseDrawer => DialogRoute::CloseDrawer,
    }
}

/// Explicit root execution plan for reconciliation coordination.
#[derive(Debug)]
enum ReconciliationRoute {
    OpenReview,
    ConfirmApplyPermanent,
    ConfirmPersistRuntime,
    BeginMutation(reconciliation::Mutation),
    FinishMutation(Result<(), BrokerError>),
    ClearRuntimeDirty,
    ConfigurationRefresh(ConfigurationEvent),
    RefreshFirewalldStatus,
    RefreshZones,
    RefreshIpSets,
    RefreshCatalogs,
}

fn plan_reconciliation_request(request: reconciliation::Request) -> ReconciliationRoute {
    match request {
        reconciliation::Request::OpenReview => ReconciliationRoute::OpenReview,
        reconciliation::Request::ConfirmApplyPermanent => {
            ReconciliationRoute::ConfirmApplyPermanent
        }
        reconciliation::Request::ConfirmPersistRuntime => {
            ReconciliationRoute::ConfirmPersistRuntime
        }
        reconciliation::Request::BeginMutation(mutation) => {
            ReconciliationRoute::BeginMutation(mutation)
        }
        reconciliation::Request::FinishMutation(result) => {
            ReconciliationRoute::FinishMutation(result)
        }
        reconciliation::Request::ClearRuntimeDirty => ReconciliationRoute::ClearRuntimeDirty,
        reconciliation::Request::ConfigurationRefresh(event) => {
            ReconciliationRoute::ConfigurationRefresh(event)
        }
        reconciliation::Request::RefreshFirewalldStatus => {
            ReconciliationRoute::RefreshFirewalldStatus
        }
        reconciliation::Request::RefreshZones => ReconciliationRoute::RefreshZones,
        reconciliation::Request::RefreshIpSets => ReconciliationRoute::RefreshIpSets,
        reconciliation::Request::RefreshCatalogs => ReconciliationRoute::RefreshCatalogs,
    }
}

/// Create a COSMIC application from the app model
impl cosmic::Application for AppModel {
    /// The async executor that will be used to run your application's commands.
    type Executor = cosmic::executor::Default;

    /// Data that your application receives to its init method.
    type Flags = ();

    /// Messages which the application and its widgets will emit.
    type Message = Message;

    /// Unique identifier in RDNN (reverse domain name notation) format.
    const APP_ID: &'static str = "dev.mmurphy.Test";

    fn core(&self) -> &cosmic::Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut cosmic::Core {
        &mut self.core
    }

    /// Initializes the application with any given flags and startup commands.
    fn init(
        core: cosmic::Core,
        _flags: Self::Flags,
    ) -> (Self, Task<cosmic::Action<Self::Message>>) {
        let navigation = navigation::State::new();

        // Create the about widget
        let about = About::default()
            .name(fl!("app-title"))
            .icon(widget::icon::from_svg_bytes(APP_ICON))
            .version(env!("CARGO_PKG_VERSION"))
            .links([(fl!("repository"), REPOSITORY)])
            .license(env!("CARGO_PKG_LICENSE"));

        // Construct the app model with the runtime's core.
        let mut app = AppModel {
            core,
            context_page: ContextPage::default(),
            about,
            navigation,
            zones: zones::State::Empty,
            reconciliation: reconciliation::State::default(),
            ipsets: ipsets::State::default(),
            dialogs: DialogState::default(),
            catalogs: catalogs::State::default(),
            operations: operations::State::new(Message::DismissToast),
            firewalld_status: FirewalldStatus::Loading,
            key_binds: HashMap::new(),
            // Optional configuration file for an application.
            config: cosmic_config::Config::new(Self::APP_ID, Config::VERSION)
                .map(|context| match Config::get_entry(&context) {
                    Ok(config) => config,
                    Err((_errors, config)) => {
                        // for why in errors {
                        //     tracing::error!(%why, "error loading app config");
                        // }

                        config
                    }
                })
                .unwrap_or_default(),
        };

        let mut outcome = outcome::Outcome::effect(app.update_title());
        outcome.append(outcome::Outcome::effect(zones::effects::start_zones_load(
            &mut app,
        )));
        outcome.append(outcome::Outcome::effect(
            zones::effects::start_firewalld_status_load(&mut app),
        ));
        let command = app.route(outcome);

        (app, command)
    }

    /// Elements to pack at the start of the header bar.
    fn header_start(&self) -> Vec<Element<'_, Self::Message>> {
        view::header_start(self)
    }

    /// Enables the COSMIC application to create a nav bar with this model.
    fn nav_model(&self) -> Option<&nav_bar::Model> {
        Some(self.navigation.nav_model())
    }

    /// The context menu to display for the active nav-bar item.
    fn nav_context_menu(&self) -> Option<Vec<menu::Tree<cosmic::Action<Self::Message>>>> {
        view::nav_context_menu(self)
    }

    /// Display a context drawer if the context page is requested.
    fn context_drawer(
        &self,
    ) -> Option<cosmic::app::context_drawer::ContextDrawer<'_, Self::Message>> {
        view::context_drawer(self)
    }

    fn dialog(&self) -> Option<Element<'_, Self::Message>> {
        view::dialog(self)
    }

    fn footer(&self) -> Option<Element<'_, Self::Message>> {
        view::footer(self)
    }

    /// Describes the interface based on the current state of the application model.
    ///
    /// Application events will be processed through the view. Any messages emitted by
    /// events received by widgets will be passed to the update method.
    fn view(&self) -> Element<'_, Self::Message> {
        view::view(self)
    }

    /// Register subscriptions for this application.
    ///
    /// Subscriptions are long-running async tasks running in the background which
    /// emit messages to the application through a channel. They can be dynamically
    /// stopped and started conditionally based on application state, or persist
    /// indefinitely.
    fn subscription(&self) -> Subscription<Self::Message> {
        // Add subscriptions which are always active.
        let mut subscriptions = vec![
            // Watch for application configuration changes.
            self.core()
                .watch_config::<Config>(Self::APP_ID)
                .map(|update| {
                    // for why in update.errors {
                    //     tracing::error!(?why, "app config error");
                    // }

                    Message::Navigation(navigation::Message::UpdateConfig(update.config))
                }),
        ];
        let selected_zone = self.current_zone_name();
        subscriptions.push(
            Subscription::run_with(
                selected_zone.clone(),
                reconciliation::configuration_event_subscription,
            )
            .map(Message::Reconciliation),
        );

        Subscription::batch(subscriptions)
    }

    /// Handles messages emitted by the application and its widgets.
    ///
    /// Tasks may be returned for asynchronous execution of code in the background
    /// on the application's async runtime.
    fn update(&mut self, message: Self::Message) -> Task<cosmic::Action<Self::Message>> {
        match message {
            Message::Navigation(message) => return self.update_navigation(message),

            Message::Dialog(dialog_message) => {
                return self.handle_dialog_message(dialog_message);
            }

            Message::Zone(zones::Message::View(action)) => {
                return self.handle_zone_action(action);
            }

            Message::IpSet(message) => {
                return self.update_ipsets(message);
            }
            Message::DismissToast(id) => {
                return self.route(outcome::Outcome::request(RootRequest::DismissToast(id)));
            }
            Message::CancelConfirmation => {
                self.operations.confirmation = None;
            }
            Message::ConfirmDestructive => {
                let Some(confirmation) = self.operations.confirmation.take() else {
                    return Task::none();
                };
                return match confirmation {
                    Confirmation::DeleteZone(zone_name) => {
                        zones::effects::start_zone_delete(self, zone_name)
                    }
                    Confirmation::DeleteIpSet(ipset_name) => {
                        self.update_ipsets(ipsets::Message::Delete(ipset_name))
                    }
                    Confirmation::StopFirewalld => {
                        zones::effects::start_firewalld_control(self, false)
                    }
                    Confirmation::ApplyPermanentConfiguration => {
                        self.handle_reconciliation_message(reconciliation::Message::ApplyPermanent)
                    }
                    Confirmation::SaveRuntimeConfiguration => {
                        self.handle_reconciliation_message(reconciliation::Message::PersistRuntime)
                    }
                };
            }
            Message::Reconciliation(message) => {
                return self.handle_reconciliation_message(message);
            }
            Message::Zone(zones::Message::FirewalldStatusLoaded(result)) => {
                self.firewalld_status = match result {
                    Ok(status) => status,
                    Err(error) => FirewalldStatus::Error(error.to_string()),
                };
                if self.firewalld_status == FirewalldStatus::Active
                    && !self.reconciliation.is_refreshing()
                {
                    if let ZoneViewState::Ready(details) = &self.zones {
                        return self.handle_reconciliation_message(reconciliation::Message::Load(
                            details.name.clone(),
                        ));
                    }
                } else {
                    self.reconciliation
                        .set_unavailable(self.current_zone_name());
                }
            }
            Message::Zone(zones::Message::DaemonControlFinished(result)) => {
                return Task::batch(vec![
                    self.finish_mutation(&result),
                    zones::effects::start_firewalld_status_load(self),
                ]);
            }

            Message::Zone(zones::Message::ListLoaded(result)) => {
                return self.update_navigation(navigation::Message::ZonesLoaded(
                    result.map_err(|error| error.to_string()),
                ));
            }
            Message::Zone(zones::Message::DetailsLoaded { zone_name, result }) => {
                let is_active = matches!(
                    self.navigation.active_item(),
                    Some(SidebarItem::Zone { name, .. }) if name == &zone_name
                );
                if !is_active {
                    return Task::none();
                }

                match *result {
                    Ok(details) => {
                        self.zones = ZoneViewState::Ready(Box::new(details));
                        if self.firewalld_status == FirewalldStatus::Active {
                            return self.handle_reconciliation_message(
                                reconciliation::Message::Load(zone_name),
                            );
                        }
                        self.reconciliation.set_unavailable(Some(zone_name));
                        return self.finish_configuration_refresh();
                    }
                    Err(error) => {
                        self.zones = ZoneViewState::Error {
                            zone: zone_name.clone(),
                            message: error.to_string(),
                        };
                        self.reconciliation.set_unavailable(Some(zone_name));
                        return self.finish_configuration_refresh();
                    }
                }
            }
            Message::Zone(zones::Message::DefaultLoaded(result)) => {
                return self.update_navigation(navigation::Message::DefaultZoneLoaded(
                    result.map_err(|error| error.to_string()),
                ));
            }
            Message::Zone(zones::Message::ActiveLoaded(result)) => {
                return self.update_navigation(navigation::Message::ActiveZonesLoaded(
                    result.map_err(|error| error.to_string()),
                ));
            }
            Message::Catalog(message) => {
                return self.update_catalogs(message);
            }
            Message::Zone(zones::Message::DefaultSet(result)) => match result {
                Ok(()) => {
                    return Task::batch(vec![
                        self.finish_mutation(&result),
                        zones::effects::start_default_zone_load(self),
                    ]);
                }
                Err(error) => {
                    return self.finish_mutation(&Err(error));
                }
            },
            Message::Zone(zones::Message::Created { zone_name, result }) => match result {
                Ok(()) => {
                    self.operations.runtime_reload_needed = true;
                    self.dialogs.reset(DialogKind::Zone);
                    self.close_context_drawer();
                    return Task::batch(vec![
                        self.finish_mutation(&result),
                        zones::effects::start_zones_load(self),
                    ]);
                }
                Err(error) => {
                    let _ = zone_name;
                    return self.finish_mutation(&Err(error));
                }
            },
            Message::Zone(zones::Message::Deleted { zone_name, result }) => match result {
                Ok(()) => {
                    self.operations.runtime_reload_needed = true;
                    if matches!(
                        self.navigation.active_item(),
                        Some(SidebarItem::Zone { name, .. }) if name == &zone_name
                    ) {
                        self.zones = ZoneViewState::Empty;
                        self.reconciliation.set_unavailable(None);
                    }
                    return Task::batch(vec![
                        self.finish_mutation(&result),
                        zones::effects::start_zones_load(self),
                    ]);
                }
                Err(error) => {
                    let _ = zone_name;
                    return self.finish_mutation(&Err(error));
                }
            },
            Message::Zone(zones::Message::ItemAdded { zone_name, result })
            | Message::Zone(zones::Message::ItemRemoved { zone_name, result }) => match result {
                Ok(()) => {
                    self.operations.runtime_reload_needed = true;
                    if self.core.window.show_context
                        && let Some(kind) = dialog_kind_for_page(self.context_page)
                    {
                        self.dialogs.reset(kind);
                        self.close_context_drawer();
                    }
                    let is_active = matches!(
                        self.navigation.active_item(),
                        Some(SidebarItem::Zone { name, .. }) if name == &zone_name
                    );
                    if is_active {
                        return Task::batch(vec![
                            self.finish_mutation(&result),
                            zones::effects::start_zone_load(self, zone_name),
                        ]);
                    }
                    return self.finish_mutation(&result);
                }
                Err(error) => {
                    let _ = zone_name;
                    return self.finish_mutation(&Err(error));
                }
            },
        }
        Task::none()
    }

    /// Called when a nav item is selected.
    fn on_nav_select(&mut self, id: nav_bar::Id) -> Task<cosmic::Action<Self::Message>> {
        self.update_navigation(navigation::Message::Select(id))
    }
}

impl AppModel {
    /// Apply root requests in FIFO order before scheduling collected effects.
    fn route(
        &mut self,
        outcome: outcome::Outcome<Task<cosmic::Action<Message>>, RootRequest>,
    ) -> Task<cosmic::Action<Message>> {
        let mut router = router::Router::new(outcome);
        while let Some(request) = router.pop_request() {
            match request {
                RootRequest::DismissToast(id) => {
                    self.operations.toasts.remove(id);
                }
            }
        }
        Task::batch(router.into_effects())
    }

    /// Delegate a catalog message and apply its root requests before starting loads.
    fn update_catalogs(&mut self, message: catalogs::Message) -> Task<cosmic::Action<Message>> {
        let selected_interface = self.dialogs.interface.interface.clone();
        let outcome = catalogs::update(
            &mut self.catalogs,
            message,
            catalogs::Context {
                selected_interface: &selected_interface,
            },
        );
        let mut router = router::Router::new(outcome);
        while let Some(request) = router.pop_request() {
            match request {
                catalogs::Request::ClearInterfaceSelection => {
                    self.dialogs.interface.interface.clear();
                    self.dialogs.interface.error = None;
                }
            }
        }
        Task::batch(router.into_effects().into_iter().map(|effect| {
            catalogs::effects(effect).map(|message| cosmic::Action::from(Message::Catalog(message)))
        }))
    }

    fn mutation_pending(&self) -> bool {
        self.operations.mutation_pending()
    }

    fn begin_mutation(&mut self, operation: String) -> bool {
        if !self.operations.begin(operation) {
            return false;
        }
        self.dialogs.operation_error = None;
        true
    }

    fn finish_mutation(
        &mut self,
        result: &Result<(), BrokerError>,
    ) -> Task<cosmic::Action<Message>> {
        let operation = self.operations.finish_label(fl!("operation-change"));
        let toast = match result {
            Ok(()) => Toast::new(fl!("operation-succeeded", operation = operation)),
            Err(error) => {
                let message = fl!(
                    "operation-failed",
                    operation = operation,
                    error = error.to_string()
                );
                if self.core.window.show_context {
                    self.dialogs.operation_error = Some(message.clone());
                }
                Toast::new(message)
            }
        };
        let toast = self.operations.push_toast(toast).map(cosmic::Action::App);
        if self.reconciliation.take_deferred_refresh() {
            return Task::batch(vec![
                toast,
                self.start_configuration_refresh(ConfigurationEvent::Reloaded),
            ]);
        }
        toast
    }

    /// Apply a semantic reconciliation message and execute any root application effect.
    fn handle_reconciliation_message(
        &mut self,
        message: reconciliation::Message,
    ) -> Task<cosmic::Action<Message>> {
        let selected_zone = self.current_zone_name();
        let ready_zone = match &self.zones {
            ZoneViewState::Ready(details) => Some(details.name.clone()),
            _ => None,
        };
        let outcome = reconciliation::update(
            &mut self.reconciliation,
            message,
            reconciliation::Context {
                selected_zone: selected_zone.as_deref(),
                ready_zone: ready_zone.as_deref(),
                firewalld_active: self.firewalld_status == FirewalldStatus::Active,
                mutation_pending: self.operations.mutation_pending(),
            },
        );
        let mut tasks = Vec::new();
        let mut router = router::Router::new(outcome);
        while let Some(request) = router.pop_request() {
            match plan_reconciliation_request(request) {
                ReconciliationRoute::OpenReview => {
                    self.open_context_page(ContextPage::ReviewReconciliation);
                }
                ReconciliationRoute::ConfirmApplyPermanent => {
                    self.operations.confirmation = Some(Confirmation::ApplyPermanentConfiguration);
                }
                ReconciliationRoute::ConfirmPersistRuntime => {
                    self.operations.confirmation = Some(Confirmation::SaveRuntimeConfiguration);
                }
                ReconciliationRoute::BeginMutation(mutation) => {
                    let operation = match mutation {
                        reconciliation::Mutation::ApplyPermanent => {
                            fl!("operation-apply-permanent")
                        }
                        reconciliation::Mutation::PersistRuntime => {
                            fl!("operation-save-runtime")
                        }
                    };
                    let _ = self.begin_mutation(operation);
                }
                ReconciliationRoute::FinishMutation(result) => {
                    tasks.push(self.finish_mutation(&result));
                }
                ReconciliationRoute::ClearRuntimeDirty => {
                    self.operations.runtime_reload_needed = false;
                }
                ReconciliationRoute::ConfigurationRefresh(event) => {
                    tasks.push(self.start_configuration_refresh(event));
                }
                ReconciliationRoute::RefreshFirewalldStatus => {
                    tasks.push(zones::effects::start_firewalld_status_load(self));
                }
                ReconciliationRoute::RefreshZones => {
                    tasks.push(zones::effects::start_zones_load(self));
                }
                ReconciliationRoute::RefreshIpSets => {
                    tasks.push(self.update_ipsets(ipsets::Message::LoadList));
                }
                ReconciliationRoute::RefreshCatalogs => {
                    tasks.extend([
                        self.update_catalogs(catalogs::Message::LoadServices),
                        self.update_catalogs(catalogs::Message::LoadIcmpTypes),
                        self.update_catalogs(catalogs::Message::LoadInterfaces),
                    ]);
                }
            }
        }
        tasks.extend(router.into_effects().into_iter().map(|effect| {
            reconciliation::effects(effect)
                .map(|message| cosmic::Action::from(Message::Reconciliation(message)))
        }));
        Task::batch(tasks)
    }

    fn start_configuration_refresh(
        &mut self,
        event: ConfigurationEvent,
    ) -> Task<cosmic::Action<Message>> {
        match event {
            ConfigurationEvent::Reloaded => Task::batch(vec![
                zones::effects::start_firewalld_status_load(self),
                zones::effects::start_zones_load(self),
                self.update_ipsets(ipsets::Message::LoadList),
                self.update_catalogs(catalogs::Message::LoadServices),
                self.update_catalogs(catalogs::Message::LoadIcmpTypes),
                self.update_catalogs(catalogs::Message::LoadInterfaces),
            ]),
            ConfigurationEvent::RuntimeZoneChanged { zone } => {
                let is_current = self.current_zone_name().as_deref() == Some(zone.as_str());
                if !is_current {
                    return self.finish_configuration_refresh();
                }
                self.handle_reconciliation_message(reconciliation::Message::Load(zone))
            }
            ConfigurationEvent::PermanentZoneUpdated { zone } => {
                let is_current = self.current_zone_name().as_deref() == Some(zone.as_str());
                if !is_current {
                    return self.finish_configuration_refresh();
                }
                zones::effects::start_zone_load(self, zone)
            }
            ConfigurationEvent::PermanentZoneRemoved { .. } => {
                zones::effects::start_zones_load(self)
            }
            ConfigurationEvent::PermanentZoneRenamed { old_zone, new_zone } => {
                self.navigation.preserve_zone_rename(&old_zone, &new_zone);
                zones::effects::start_zones_load(self)
            }
        }
    }

    fn finish_configuration_refresh(&mut self) -> Task<cosmic::Action<Message>> {
        let mut router =
            router::Router::new(reconciliation::finish_refresh(&mut self.reconciliation));
        let mut tasks = Vec::new();
        while let Some(request) = router.pop_request() {
            if let reconciliation::Request::ConfigurationRefresh(event) = request {
                tasks.push(self.start_configuration_refresh(event));
            }
        }
        debug_assert!(router.into_effects().is_empty());
        Task::batch(tasks)
    }

    fn open_context_page(&mut self, context_page: ContextPage) {
        self.context_page = context_page;
        self.core.window.show_context = true;
        self.reset_dialog_for_context(context_page);
    }

    fn reset_dialog_for_context(&mut self, context_page: ContextPage) {
        if let Some(kind) = dialog_kind_for_page(context_page) {
            self.dialogs.reset(kind);
        }
    }

    fn close_context_drawer(&mut self) {
        self.core.window.show_context = false;
    }

    fn current_zone_name(&self) -> Option<String> {
        match &self.zones {
            ZoneViewState::Ready(details) => Some(details.name.clone()),
            _ => None,
        }
    }

    /// Reduce navigation state and execute its root-owned requests in FIFO order.
    fn update_navigation(&mut self, message: navigation::Message) -> Task<cosmic::Action<Message>> {
        let outcome = navigation::update(&mut self.navigation, message, navigation::Context);
        let mut tasks = Vec::new();
        let mut router = router::Router::new(outcome);
        while let Some(request) = router.pop_request() {
            match request {
                navigation::Request::LoadZone(zone_name) => {
                    tasks.push(zones::effects::start_zone_load(self, zone_name));
                }
                navigation::Request::LoadIpSets => {
                    tasks.push(self.update_ipsets(ipsets::Message::LoadList));
                }
                navigation::Request::OpenContextPage(context_page) => {
                    self.open_context_page(context_page);
                }
                navigation::Request::ToggleContextPage(context_page) => {
                    let requires_zone = matches!(
                        context_page,
                        ContextPage::AddService
                            | ContextPage::AddPort
                            | ContextPage::AddInterface
                            | ContextPage::AddSource
                            | ContextPage::AddIcmp
                            | ContextPage::AddRichRule
                    );
                    if self.context_page == context_page {
                        self.core.window.show_context = !self.core.window.show_context;
                    } else {
                        self.context_page = context_page;
                        self.core.window.show_context = true;
                    }
                    if self.core.window.show_context {
                        self.reset_dialog_for_context(context_page);
                        if requires_zone && self.current_zone_name().is_none() {
                            self.dialogs.operation_error = Some(fl!("error-select-zone-first"));
                        }
                        match context_page {
                            ContextPage::AddInterface => {
                                tasks.push(self.update_catalogs(catalogs::Message::LoadInterfaces));
                            }
                            ContextPage::AddService => {
                                tasks.push(self.update_catalogs(catalogs::Message::LoadServices));
                            }
                            ContextPage::AddIcmp => {
                                tasks.push(self.update_catalogs(catalogs::Message::LoadIcmpTypes));
                            }
                            _ => {}
                        }
                    }
                }
                navigation::Request::LoadInterfaceCatalog => {
                    tasks.push(self.update_catalogs(catalogs::Message::LoadInterfaces));
                }
                navigation::Request::SetDefaultZone(zone_name) => {
                    tasks.push(zones::effects::start_default_zone_set(self, zone_name));
                }
                navigation::Request::ConfirmDeleteZone(zone_name) => {
                    self.operations.confirmation = Some(Confirmation::DeleteZone(zone_name));
                }
                navigation::Request::RefreshTitle => tasks.push(self.update_title()),
                navigation::Request::ClearSelectedZone => {
                    self.zones = ZoneViewState::Empty;
                    self.reconciliation.set_unavailable(None);
                }
                navigation::Request::LoadDefaultZone => {
                    tasks.push(zones::effects::start_default_zone_load(self));
                }
                navigation::Request::LoadActiveZones => {
                    tasks.push(zones::effects::start_active_zones_load(self));
                }
                navigation::Request::FinishConfigurationRefresh => {
                    tasks.push(self.finish_configuration_refresh());
                }
                navigation::Request::ShowZoneListError(message) => {
                    self.zones = ZoneViewState::Error {
                        zone: "zones".to_string(),
                        message,
                    };
                }
                navigation::Request::UpdateConfig(config) => self.config = config,
                navigation::Request::LaunchUrl(url) => {
                    if let Err(error) = open::that_detached(&url) {
                        eprintln!("failed to open {url:?}: {error}");
                    }
                }
            }
        }
        debug_assert!(router.into_effects().is_empty());
        Task::batch(tasks)
    }

    fn handle_zone_action(&mut self, action: ZoneViewAction) -> Task<cosmic::Action<Message>> {
        if self.mutation_pending() {
            return Task::none();
        }
        match &action {
            ZoneViewAction::Reconciliation(action) => {
                return self.handle_reconciliation_action(*action);
            }
            ZoneViewAction::AddService => {
                self.open_context_page(ContextPage::AddService);
                return self.update_catalogs(catalogs::Message::LoadServices);
            }
            ZoneViewAction::SetMasquerade(enabled) => {
                let Some(zone_name) = self.current_zone_name() else {
                    return Task::none();
                };
                return zones::effects::start_masquerade_set(self, zone_name, *enabled);
            }
            ZoneViewAction::SetIcmpBlockInversion(enabled) => {
                let Some(zone_name) = self.current_zone_name() else {
                    return Task::none();
                };
                return zones::effects::start_icmp_inversion_set(self, zone_name, *enabled);
            }
            ZoneViewAction::StartFirewalld => {
                return zones::effects::start_firewalld_control(self, true);
            }
            ZoneViewAction::StopFirewalld => {
                self.operations.confirmation = Some(Confirmation::StopFirewalld);
                return Task::none();
            }
            ZoneViewAction::AddInterface => {
                self.open_context_page(ContextPage::AddInterface);
                return self.update_catalogs(catalogs::Message::LoadInterfaces);
            }
            ZoneViewAction::AddPort { kind } => {
                self.open_context_page(ContextPage::AddPort);
                self.dialogs.port.kind = *kind;
                return Task::none();
            }
            ZoneViewAction::AddSource => {
                self.open_context_page(ContextPage::AddSource);
                return Task::none();
            }
            ZoneViewAction::AddIcmpBlock => {
                self.open_context_page(ContextPage::AddIcmp);
                return self.update_catalogs(catalogs::Message::LoadIcmpTypes);
            }
            ZoneViewAction::AddRichRule => {
                self.open_context_page(ContextPage::AddRichRule);
                return Task::none();
            }
            ZoneViewAction::RemoveService(_)
            | ZoneViewAction::RemoveInterface(_)
            | ZoneViewAction::RemoveSource(_)
            | ZoneViewAction::RemovePort { .. }
            | ZoneViewAction::RemoveForwardPort { .. }
            | ZoneViewAction::RemoveSourcePort { .. }
            | ZoneViewAction::RemoveIcmpBlock(_)
            | ZoneViewAction::RemoveRichRule(_) => {}
        }

        let zone_name = match &self.zones {
            ZoneViewState::Ready(details) => details.name.clone(),
            _ => return Task::none(),
        };

        zones::effects::start_zone_item_remove(self, zone_name, action)
    }

    /// Route actions shared by the reconciliation banner and review drawer.
    fn handle_reconciliation_action(
        &mut self,
        action: reconciliation::ReconciliationAction,
    ) -> Task<cosmic::Action<Message>> {
        self.handle_reconciliation_message(reconciliation::Message::Action(action))
    }

    fn handle_dialog_message(&mut self, message: DialogMessage) -> Task<cosmic::Action<Message>> {
        let selected_zone = self.current_zone_name();
        let (enabled_services, blocked_icmp) = match &self.zones {
            ZoneViewState::Ready(details) => {
                (details.services.as_slice(), details.icmp_blocks.as_slice())
            }
            _ => (&[][..], &[][..]),
        };
        let outcome = dialogs::update(
            &mut self.dialogs,
            message,
            dialogs::Context {
                selected_zone: selected_zone.as_deref(),
                interfaces: self.catalogs.interfaces.items(),
                enabled_services,
                blocked_icmp,
                mutation_pending: self.operations.mutation_pending(),
            },
        );
        let mut tasks = Vec::new();
        let mut router = router::Router::new(outcome);
        while let Some(request) = router.pop_request() {
            match plan_dialog_request(request) {
                DialogRoute::CreateZone {
                    name,
                    description,
                    target,
                } => {
                    tasks.push(zones::effects::start_zone_create(
                        self,
                        name,
                        description,
                        target,
                    ));
                }
                DialogRoute::AddService { zone, service } => {
                    tasks.push(zones::effects::start_service_add(self, zone, service));
                }
                DialogRoute::AddPort {
                    zone,
                    port,
                    protocol,
                } => {
                    tasks.push(zones::effects::start_port_add(self, zone, port, protocol));
                }
                DialogRoute::AddSourcePort {
                    zone,
                    port,
                    protocol,
                } => {
                    tasks.push(zones::effects::start_source_port_add(
                        self, zone, port, protocol,
                    ));
                }
                DialogRoute::AddForwardPort {
                    zone,
                    port,
                    protocol,
                    to_port,
                    to_addr,
                } => {
                    tasks.push(zones::effects::start_forward_port_add(
                        self, zone, port, protocol, to_port, to_addr,
                    ));
                }
                DialogRoute::AddInterface { zone, interface } => {
                    tasks.push(zones::effects::start_interface_add(self, zone, interface));
                }
                DialogRoute::AddSource { zone, source } => {
                    tasks.push(zones::effects::start_source_add(self, zone, source));
                }
                DialogRoute::AddIcmp { zone, icmp } => {
                    tasks.push(zones::effects::start_icmp_add(self, zone, icmp));
                }
                DialogRoute::AddRichRule { zone, rule } => {
                    tasks.push(zones::effects::start_rich_rule_add(self, zone, rule));
                }
                DialogRoute::CreateIpSet {
                    name,
                    ipset_type,
                    entries,
                } => {
                    tasks.push(self.start_ipset_create(name, ipset_type, entries));
                }
                DialogRoute::CloseDrawer => self.close_context_drawer(),
            }
        }
        tasks.extend(router.into_effects().into_iter().map(|effect| {
            dialogs::effects(effect).map(|message| cosmic::Action::from(Message::Dialog(message)))
        }));
        Task::batch(tasks)
    }

    /// Delegate an IP-set message and process root requests before slice effects.
    fn update_ipsets(&mut self, message: ipsets::Message) -> Task<cosmic::Action<Message>> {
        let outcome = ipsets::update(
            &mut self.ipsets,
            message,
            ipsets::Context {
                mutation_pending: self.operations.mutation_pending(),
                localize_validation: localized_validation_error,
            },
        );
        let mut tasks = Vec::new();
        let mut router = router::Router::new(outcome);
        while let Some(request) = router.pop_request() {
            match request {
                ipsets::Request::BeginMutation(mutation) => {
                    let operation = match mutation {
                        ipsets::Mutation::AddEntry => fl!("operation-add-ipset-entry"),
                        ipsets::Mutation::RemoveEntry => fl!("operation-remove-ipset-entry"),
                        ipsets::Mutation::Create => fl!("operation-create-ipset"),
                        ipsets::Mutation::Delete => fl!("operation-delete-ipset"),
                    };
                    let _ = self.begin_mutation(operation);
                }
                ipsets::Request::ConfirmDelete(ipset_name) => {
                    self.operations.confirmation = Some(Confirmation::DeleteIpSet(ipset_name));
                }
                ipsets::Request::MarkRuntimeDirty => {
                    self.operations.runtime_reload_needed = true;
                }
                ipsets::Request::FinishMutation(result) => {
                    tasks.push(self.finish_mutation(&result));
                }
                ipsets::Request::ResetCreateDialog => {
                    self.dialogs.reset(DialogKind::IpSet);
                }
                ipsets::Request::CloseDrawer => {
                    self.close_context_drawer();
                }
            }
        }
        tasks.extend(router.into_effects().into_iter().map(|effect| {
            ipsets::effects(effect).map(|message| cosmic::Action::from(Message::IpSet(message)))
        }));
        Task::batch(tasks)
    }

    fn start_ipset_create(
        &mut self,
        ipset_name: String,
        ipset_type: String,
        entries: Vec<String>,
    ) -> Task<cosmic::Action<Message>> {
        self.update_ipsets(ipsets::Message::Create {
            name: ipset_name,
            ipset_type,
            entries,
        })
    }

    /// Updates the header and window titles.
    pub fn update_title(&mut self) -> Task<cosmic::Action<Message>> {
        let mut window_title = fl!("app-title");

        if let Some(page) = self.navigation.active_label() {
            window_title.push_str(" — ");
            window_title.push_str(&page);
        }

        if let Some(id) = self.core.main_window_id() {
            self.set_window_title(window_title, id)
        } else {
            Task::none()
        }
    }
}

/// The context page to display in the context drawer.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum ContextPage {
    #[default]
    About,
    AddZone,
    AddService,
    AddPort,
    AddInterface,
    AddSource,
    AddIcmp,
    AddRichRule,
    AddIpSet,
    ReviewReconciliation,
}

fn dialog_kind_for_page(page: ContextPage) -> Option<DialogKind> {
    match page {
        ContextPage::AddZone => Some(DialogKind::Zone),
        ContextPage::AddService => Some(DialogKind::Service),
        ContextPage::AddPort => Some(DialogKind::Port),
        ContextPage::AddInterface => Some(DialogKind::Interface),
        ContextPage::AddSource => Some(DialogKind::Source),
        ContextPage::AddIcmp => Some(DialogKind::Icmp),
        ContextPage::AddRichRule => Some(DialogKind::RichRule),
        ContextPage::AddIpSet => Some(DialogKind::IpSet),
        ContextPage::About | ContextPage::ReviewReconciliation => None,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MenuAction {
    About,
    AddZone,
    AddPort,
    AddInterface,
    AddSource,
    AddIcmp,
    AddRichRule,
    AddIpSet,
}

impl menu::action::MenuAction for MenuAction {
    type Message = Message;

    fn message(&self) -> Self::Message {
        match self {
            MenuAction::About => {
                Message::Navigation(navigation::Message::ToggleContextPage(ContextPage::About))
            }
            MenuAction::AddZone => {
                Message::Navigation(navigation::Message::ToggleContextPage(ContextPage::AddZone))
            }
            MenuAction::AddPort => {
                Message::Navigation(navigation::Message::ToggleContextPage(ContextPage::AddPort))
            }
            MenuAction::AddInterface => Message::Navigation(
                navigation::Message::ToggleContextPage(ContextPage::AddInterface),
            ),
            MenuAction::AddSource => Message::Navigation(navigation::Message::ToggleContextPage(
                ContextPage::AddSource,
            )),
            MenuAction::AddIcmp => {
                Message::Navigation(navigation::Message::ToggleContextPage(ContextPage::AddIcmp))
            }
            MenuAction::AddRichRule => Message::Navigation(navigation::Message::ToggleContextPage(
                ContextPage::AddRichRule,
            )),
            MenuAction::AddIpSet => Message::Navigation(navigation::Message::ToggleContextPage(
                ContextPage::AddIpSet,
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::RefreshRequest;

    fn dialog_context(mutation_pending: bool) -> dialogs::Context<'static> {
        dialogs::Context {
            selected_zone: Some("public"),
            interfaces: &[],
            enabled_services: &[],
            blocked_icmp: &[],
            mutation_pending,
        }
    }

    #[test]
    fn dialog_submissions_plan_the_matching_domain_effects() {
        let routes = [
            plan_dialog_request(dialogs::Request::Submit(dialogs::Submission::Zone {
                name: "work".into(),
                description: "Work".into(),
                target: crate::models::ZoneTarget::Drop,
            })),
            plan_dialog_request(dialogs::Request::Submit(dialogs::Submission::Service {
                zone: "public".into(),
                service: "ssh".into(),
            })),
            plan_dialog_request(dialogs::Request::Submit(dialogs::Submission::Port {
                zone: "public".into(),
                port: "443".into(),
                protocol: "tcp".into(),
            })),
            plan_dialog_request(dialogs::Request::Submit(dialogs::Submission::SourcePort {
                zone: "public".into(),
                port: "5353".into(),
                protocol: "udp".into(),
            })),
            plan_dialog_request(dialogs::Request::Submit(dialogs::Submission::ForwardPort {
                zone: "public".into(),
                port: "443".into(),
                protocol: "tcp".into(),
                to_port: "8443".into(),
                to_addr: "192.0.2.2".into(),
            })),
            plan_dialog_request(dialogs::Request::Submit(dialogs::Submission::Interface {
                zone: "public".into(),
                interface: "eth0".into(),
            })),
            plan_dialog_request(dialogs::Request::Submit(dialogs::Submission::Source {
                zone: "public".into(),
                source: "192.0.2.0/24".into(),
            })),
            plan_dialog_request(dialogs::Request::Submit(dialogs::Submission::Icmp {
                zone: "public".into(),
                icmp: "echo-request".into(),
            })),
            plan_dialog_request(dialogs::Request::Submit(dialogs::Submission::RichRule {
                zone: "public".into(),
                rule: "<rule><accept/></rule>".into(),
            })),
            plan_dialog_request(dialogs::Request::Submit(dialogs::Submission::IpSet {
                name: "trusted".into(),
                ipset_type: "hash:ip".into(),
                entries: vec!["192.0.2.1".into()],
            })),
        ];

        assert!(matches!(routes[0], DialogRoute::CreateZone { .. }));
        assert!(matches!(routes[1], DialogRoute::AddService { .. }));
        assert!(matches!(routes[2], DialogRoute::AddPort { .. }));
        assert!(matches!(routes[3], DialogRoute::AddSourcePort { .. }));
        assert!(matches!(routes[4], DialogRoute::AddForwardPort { .. }));
        assert!(matches!(routes[5], DialogRoute::AddInterface { .. }));
        assert!(matches!(routes[6], DialogRoute::AddSource { .. }));
        assert!(matches!(routes[7], DialogRoute::AddIcmp { .. }));
        assert!(matches!(routes[8], DialogRoute::AddRichRule { .. }));
        assert!(matches!(routes[9], DialogRoute::CreateIpSet { .. }));
    }

    #[test]
    fn dialog_cancellation_and_pending_mutations_are_routed_safely() {
        assert_eq!(
            plan_dialog_request(dialogs::Request::CloseDrawer),
            DialogRoute::CloseDrawer
        );

        let mut state = DialogState::default();
        state.zone.name = "work".into();
        let outcome = dialogs::update(
            &mut state,
            DialogMessage::Submit(DialogKind::Zone),
            dialog_context(true),
        );
        assert!(outcome.requests.is_empty());
    }

    #[test]
    fn reconciliation_root_routes_keep_confirmations_and_refreshes_distinct() {
        assert!(matches!(
            plan_reconciliation_request(reconciliation::Request::OpenReview),
            ReconciliationRoute::OpenReview
        ));
        assert!(matches!(
            plan_reconciliation_request(reconciliation::Request::ConfirmApplyPermanent),
            ReconciliationRoute::ConfirmApplyPermanent
        ));
        assert!(matches!(
            plan_reconciliation_request(reconciliation::Request::ConfirmPersistRuntime),
            ReconciliationRoute::ConfirmPersistRuntime
        ));

        let mut router = router::Router::new(outcome::Outcome::<(), _> {
            effects: Vec::new(),
            requests: vec![
                reconciliation::Request::RefreshFirewalldStatus,
                reconciliation::Request::RefreshZones,
                reconciliation::Request::RefreshIpSets,
                reconciliation::Request::RefreshCatalogs,
            ],
        });
        assert!(matches!(
            plan_reconciliation_request(router.pop_request().expect("status refresh")),
            ReconciliationRoute::RefreshFirewalldStatus
        ));
        assert!(matches!(
            plan_reconciliation_request(router.pop_request().expect("zone refresh")),
            ReconciliationRoute::RefreshZones
        ));
        assert!(matches!(
            plan_reconciliation_request(router.pop_request().expect("IP-set refresh")),
            ReconciliationRoute::RefreshIpSets
        ));
        assert!(matches!(
            plan_reconciliation_request(router.pop_request().expect("catalog refresh")),
            ReconciliationRoute::RefreshCatalogs
        ));
        assert!(router.pop_request().is_none());
        assert!(router.into_effects().is_empty());
    }

    #[test]
    fn deferred_configuration_refresh_is_released_after_mutation_work() {
        let mut state = reconciliation::State::default();
        state.selection_changed(Some("public".into()));

        assert_eq!(
            state.handle_configuration_event(true),
            RefreshRequest::Coalesced
        );
        assert!(state.take_deferred_refresh());
        assert!(!state.take_deferred_refresh());
    }
}
