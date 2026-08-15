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
/// Pure root policy for one firewalld configuration event.
#[derive(Debug, Clone, Eq, PartialEq)]
enum ConfigurationRefreshPlan {
    ReloadEverything,
    ReloadCurrentReconciliation { zone: String },
    ReloadCurrentZone { zone: String },
    ReloadZones,
    PreserveRenameAndReloadZones { old_zone: String, new_zone: String },
    FinishWithoutReload,
}

/// Minimum immutable root context needed to classify a configuration event.
#[derive(Clone, Copy, Debug)]
struct ConfigurationRefreshContext<'a> {
    selected_zone: Option<&'a str>,
}

fn plan_configuration_refresh(
    event: ConfigurationEvent,
    context: ConfigurationRefreshContext<'_>,
) -> ConfigurationRefreshPlan {
    match event {
        ConfigurationEvent::Reloaded => ConfigurationRefreshPlan::ReloadEverything,
        ConfigurationEvent::RuntimeZoneChanged { zone } => {
            if context.selected_zone == Some(zone.as_str()) {
                ConfigurationRefreshPlan::ReloadCurrentReconciliation { zone }
            } else {
                ConfigurationRefreshPlan::FinishWithoutReload
            }
        }
        ConfigurationEvent::PermanentZoneUpdated { zone } => {
            if context.selected_zone == Some(zone.as_str()) {
                ConfigurationRefreshPlan::ReloadCurrentZone { zone }
            } else {
                ConfigurationRefreshPlan::FinishWithoutReload
            }
        }
        ConfigurationEvent::PermanentZoneRemoved { .. } => ConfigurationRefreshPlan::ReloadZones,
        ConfigurationEvent::PermanentZoneRenamed { old_zone, new_zone } => {
            ConfigurationRefreshPlan::PreserveRenameAndReloadZones { old_zone, new_zone }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ReloadEverythingStep {
    FirewalldStatus,
    Zones,
    IpSets,
    Services,
    IcmpTypes,
    Interfaces,
}

const RELOAD_EVERYTHING_STEPS: [ReloadEverythingStep; 6] = [
    ReloadEverythingStep::FirewalldStatus,
    ReloadEverythingStep::Zones,
    ReloadEverythingStep::IpSets,
    ReloadEverythingStep::Services,
    ReloadEverythingStep::IcmpTypes,
    ReloadEverythingStep::Interfaces,
];

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
            zones: zones::State::default(),
            reconciliation: reconciliation::State::default(),
            ipsets: ipsets::State::default(),
            dialogs: DialogState::default(),
            catalogs: catalogs::State::default(),
            operations: operations::State::new(Message::DismissToast),
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
        outcome.append(outcome::Outcome::effect(
            app.update_zones(zones::Message::LoadList),
        ));
        outcome.append(outcome::Outcome::effect(
            app.update_zones(zones::Message::LoadStatus),
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

            Message::Zone(message) => return self.update_zones(message),

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
                        self.update_zones(zones::Message::Delete(zone_name))
                    }
                    Confirmation::DeleteIpSet(ipset_name) => {
                        self.update_ipsets(ipsets::Message::Delete(ipset_name))
                    }
                    Confirmation::StopFirewalld => {
                        self.update_zones(zones::Message::ControlFirewalld(false))
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
            Message::Catalog(message) => {
                return self.update_catalogs(message);
            }
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
        let ready_zone = self.zones.current_zone_name().map(str::to_string);
        let outcome = reconciliation::update(
            &mut self.reconciliation,
            message,
            reconciliation::Context {
                selected_zone: selected_zone.as_deref(),
                ready_zone: ready_zone.as_deref(),
                firewalld_active: self.zones.firewalld_status() == &FirewalldStatus::Active,
                mutation_pending: self.operations.mutation_pending(),
            },
        );
        let mut tasks = Vec::new();
        let mut router = router::Router::new(outcome);
        while let Some(request) = router.pop_request() {
            match request {
                reconciliation::Request::OpenReview => {
                    tasks.push(self.open_context_page(ContextPage::ReviewReconciliation));
                }
                reconciliation::Request::ConfirmApplyPermanent => {
                    self.operations.confirmation = Some(Confirmation::ApplyPermanentConfiguration);
                }
                reconciliation::Request::ConfirmPersistRuntime => {
                    self.operations.confirmation = Some(Confirmation::SaveRuntimeConfiguration);
                }
                reconciliation::Request::BeginMutation(mutation) => {
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
                reconciliation::Request::FinishMutation(result) => {
                    tasks.push(self.finish_mutation(&result));
                }
                reconciliation::Request::ClearRuntimeDirty => {
                    self.operations.runtime_reload_needed = false;
                }
                reconciliation::Request::ConfigurationRefresh(event) => {
                    tasks.push(self.start_configuration_refresh(event));
                }
                reconciliation::Request::RefreshFirewalldStatus => {
                    tasks.push(self.update_zones(zones::Message::LoadStatus));
                }
                reconciliation::Request::RefreshZones => {
                    tasks.push(self.update_zones(zones::Message::LoadList));
                }
                reconciliation::Request::RefreshIpSets => {
                    tasks.push(self.update_ipsets(ipsets::Message::LoadList));
                }
                reconciliation::Request::RefreshCatalogs => {
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
        let selected_zone = self.current_zone_name();
        match plan_configuration_refresh(
            event,
            ConfigurationRefreshContext {
                selected_zone: selected_zone.as_deref(),
            },
        ) {
            ConfigurationRefreshPlan::ReloadEverything => self.reload_everything(),
            ConfigurationRefreshPlan::ReloadCurrentReconciliation { zone } => {
                self.handle_reconciliation_message(reconciliation::Message::Load(zone))
            }
            ConfigurationRefreshPlan::ReloadCurrentZone { zone } => {
                self.update_zones(zones::Message::LoadDetails(zone))
            }
            ConfigurationRefreshPlan::ReloadZones => self.update_zones(zones::Message::LoadList),
            ConfigurationRefreshPlan::PreserveRenameAndReloadZones { old_zone, new_zone } => {
                self.navigation.preserve_zone_rename(&old_zone, &new_zone);
                self.update_zones(zones::Message::LoadList)
            }
            ConfigurationRefreshPlan::FinishWithoutReload => self.finish_configuration_refresh(),
        }
    }

    fn reload_everything(&mut self) -> Task<cosmic::Action<Message>> {
        Task::batch(RELOAD_EVERYTHING_STEPS.map(|step| match step {
            ReloadEverythingStep::FirewalldStatus => self.update_zones(zones::Message::LoadStatus),
            ReloadEverythingStep::Zones => self.update_zones(zones::Message::LoadList),
            ReloadEverythingStep::IpSets => self.update_ipsets(ipsets::Message::LoadList),
            ReloadEverythingStep::Services => self.update_catalogs(catalogs::Message::LoadServices),
            ReloadEverythingStep::IcmpTypes => {
                self.update_catalogs(catalogs::Message::LoadIcmpTypes)
            }
            ReloadEverythingStep::Interfaces => {
                self.update_catalogs(catalogs::Message::LoadInterfaces)
            }
        }))
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

    fn open_context_page(&mut self, context_page: ContextPage) -> Task<cosmic::Action<Message>> {
        self.context_page = context_page;
        self.core.window.show_context = true;
        self.reset_dialog_for_context(context_page);
        self.load_context_catalog(context_page)
    }

    fn reset_dialog_for_context(&mut self, context_page: ContextPage) {
        if let Some(kind) = context_page.descriptor().dialog {
            self.dialogs.reset(kind);
        }
    }

    fn load_context_catalog(&mut self, context_page: ContextPage) -> Task<cosmic::Action<Message>> {
        match context_page.descriptor().catalog {
            Some(ContextCatalog::Services) => self.update_catalogs(catalogs::Message::LoadServices),
            Some(ContextCatalog::Interfaces) => {
                self.update_catalogs(catalogs::Message::LoadInterfaces)
            }
            Some(ContextCatalog::IcmpTypes) => {
                self.update_catalogs(catalogs::Message::LoadIcmpTypes)
            }
            None => Task::none(),
        }
    }

    fn close_context_drawer(&mut self) {
        self.core.window.show_context = false;
    }

    fn current_zone_name(&self) -> Option<String> {
        self.zones.current_zone_name().map(str::to_string)
    }

    /// Reduce zone state, apply root requests FIFO, then schedule zone effects.
    fn update_zones(&mut self, message: zones::Message) -> Task<cosmic::Action<Message>> {
        let selected_zone = match self.navigation.active_item() {
            Some(SidebarItem::Zone { name, .. }) => Some(name.clone()),
            _ => None,
        };
        let open_dialog = if self.core.window.show_context {
            self.context_page.descriptor().dialog
        } else {
            None
        };
        let outcome = zones::update(
            &mut self.zones,
            message,
            zones::Context {
                mutation_pending: self.operations.mutation_pending(),
                selected_zone: selected_zone.as_deref(),
                reconciliation_refreshing: self.reconciliation.is_refreshing(),
                open_dialog,
            },
        );
        let mut tasks = Vec::new();
        let mut router = router::Router::new(outcome);
        while let Some(request) = router.pop_request() {
            match request {
                zones::Request::NavigationLoading => self.navigation.set_loading(),
                zones::Request::NavigationZonesLoaded(result) => {
                    tasks.push(self.update_navigation(navigation::Message::ZonesLoaded(result)));
                }
                zones::Request::NavigationDefaultLoaded(result) => tasks
                    .push(self.update_navigation(navigation::Message::DefaultZoneLoaded(result))),
                zones::Request::NavigationActiveLoaded(result) => tasks
                    .push(self.update_navigation(navigation::Message::ActiveZonesLoaded(result))),
                zones::Request::OpenContextPage(page) => {
                    tasks.push(self.open_context_page(page));
                }
                zones::Request::SetPortKind(kind) => self.dialogs.port.kind = kind,
                zones::Request::ResetDialog(kind) => self.dialogs.reset(kind),
                zones::Request::ReconciliationSelectionChanged(zone) => {
                    self.reconciliation.selection_changed(zone);
                }
                zones::Request::LoadReconciliation(zone) => tasks
                    .push(self.handle_reconciliation_message(reconciliation::Message::Load(zone))),
                zones::Request::ReconciliationUnavailable(zone) => {
                    self.reconciliation.set_unavailable(zone);
                }
                zones::Request::ReconciliationAction(action) => {
                    tasks.push(self.handle_reconciliation_action(action));
                }
                zones::Request::FinishConfigurationRefresh => {
                    tasks.push(self.finish_configuration_refresh());
                }
                zones::Request::ConfirmDeleteZone(zone) => {
                    self.operations.confirmation = Some(Confirmation::DeleteZone(zone));
                }
                zones::Request::ConfirmStopFirewalld => {
                    self.operations.confirmation = Some(Confirmation::StopFirewalld);
                }
                zones::Request::BeginMutation(mutation) => {
                    let operation = match mutation {
                        zones::Mutation::CreateZone => fl!("operation-create-zone"),
                        zones::Mutation::DeleteZone => fl!("operation-delete-zone"),
                        zones::Mutation::AddService => fl!("operation-add-service"),
                        zones::Mutation::AddPort => fl!("operation-add-port"),
                        zones::Mutation::AddSourcePort => fl!("operation-add-source-port"),
                        zones::Mutation::AddForwardPort => fl!("operation-add-forward-port"),
                        zones::Mutation::AddInterface => fl!("operation-add-interface"),
                        zones::Mutation::AddSource => fl!("operation-add-source"),
                        zones::Mutation::AddIcmp => fl!("operation-add-icmp"),
                        zones::Mutation::AddRichRule => fl!("operation-add-rich-rule"),
                        zones::Mutation::RemoveItem => fl!("operation-remove-zone-item"),
                        zones::Mutation::SetMasquerade => fl!("operation-set-masquerading"),
                        zones::Mutation::SetIcmpBlockInversion => {
                            fl!("operation-set-icmp-inversion")
                        }
                        zones::Mutation::SetDefaultZone => fl!("operation-set-default-zone"),
                        zones::Mutation::StartFirewalld => fl!("operation-start-firewalld"),
                        zones::Mutation::StopFirewalld => fl!("operation-stop-firewalld"),
                    };
                    let _ = self.begin_mutation(operation);
                }
                zones::Request::FinishMutation(result) => {
                    tasks.push(self.finish_mutation(&result));
                }
                zones::Request::MarkRuntimeDirty => {
                    self.operations.runtime_reload_needed = true;
                }
                zones::Request::CloseDrawer => self.close_context_drawer(),
                zones::Request::RefreshZones => {
                    tasks.push(self.update_zones(zones::Message::LoadList));
                }
                zones::Request::RefreshDefault => {
                    tasks.push(self.update_zones(zones::Message::LoadDefault));
                }
                zones::Request::RefreshStatus => {
                    tasks.push(self.update_zones(zones::Message::LoadStatus));
                }
                zones::Request::RefreshCurrentZone(zone) => {
                    tasks.push(self.update_zones(zones::Message::LoadDetails(zone)));
                }
            }
        }
        tasks.extend(router.into_effects().into_iter().map(|effect| {
            zones::effects(effect).map(|message| cosmic::Action::from(Message::Zone(message)))
        }));
        Task::batch(tasks)
    }

    /// Reduce navigation state and execute its root-owned requests in FIFO order.
    fn update_navigation(&mut self, message: navigation::Message) -> Task<cosmic::Action<Message>> {
        let outcome = navigation::update(&mut self.navigation, message, navigation::Context);
        let mut tasks = Vec::new();
        let mut router = router::Router::new(outcome);
        while let Some(request) = router.pop_request() {
            match request {
                navigation::Request::LoadZone(zone_name) => {
                    tasks.push(self.update_zones(zones::Message::LoadDetails(zone_name)));
                }
                navigation::Request::LoadIpSets => {
                    tasks.push(self.update_ipsets(ipsets::Message::LoadList));
                }
                navigation::Request::OpenContextPage(context_page) => {
                    tasks.push(self.open_context_page(context_page));
                }
                navigation::Request::ToggleContextPage(context_page) => {
                    let descriptor = context_page.descriptor();
                    if self.context_page == context_page {
                        self.core.window.show_context = !self.core.window.show_context;
                    } else {
                        self.context_page = context_page;
                        self.core.window.show_context = true;
                    }
                    if self.core.window.show_context {
                        self.reset_dialog_for_context(context_page);
                        if descriptor.requires_zone && self.current_zone_name().is_none() {
                            self.dialogs.operation_error = Some(fl!("error-select-zone-first"));
                        }
                        tasks.push(self.load_context_catalog(context_page));
                    }
                }
                navigation::Request::SetDefaultZone(zone_name) => {
                    tasks.push(self.update_zones(zones::Message::SetDefault(zone_name)));
                }
                navigation::Request::ConfirmDeleteZone(zone_name) => {
                    tasks.push(self.update_zones(zones::Message::ConfirmDelete(zone_name)));
                }
                navigation::Request::RefreshTitle => tasks.push(self.update_title()),
                navigation::Request::ClearSelectedZone => {
                    tasks.push(self.update_zones(zones::Message::ClearSelection));
                    self.reconciliation.set_unavailable(None);
                }
                navigation::Request::LoadDefaultZone => {
                    tasks.push(self.update_zones(zones::Message::LoadDefault));
                }
                navigation::Request::LoadActiveZones => {
                    tasks.push(self.update_zones(zones::Message::LoadActive));
                }
                navigation::Request::FinishConfigurationRefresh => {
                    tasks.push(self.finish_configuration_refresh());
                }
                navigation::Request::ShowZoneListError(message) => {
                    tasks.push(self.update_zones(zones::Message::ShowListError(message)));
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

    /// Route actions shared by the reconciliation banner and review drawer.
    fn handle_reconciliation_action(
        &mut self,
        action: reconciliation::ReconciliationAction,
    ) -> Task<cosmic::Action<Message>> {
        self.handle_reconciliation_message(reconciliation::Message::Action(action))
    }

    fn handle_dialog_message(&mut self, message: DialogMessage) -> Task<cosmic::Action<Message>> {
        let selected_zone = self.current_zone_name();
        let (enabled_services, blocked_icmp) = self
            .zones
            .ready_detail()
            .map(|details| (details.services.as_slice(), details.icmp_blocks.as_slice()))
            .unwrap_or((&[][..], &[][..]));
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
            match request {
                dialogs::Request::Submit(submission) => match submission {
                    dialogs::Submission::Zone {
                        name,
                        description,
                        target,
                    } => {
                        tasks.push(self.update_zones(zones::Message::Create {
                            name,
                            description,
                            target,
                        }));
                    }
                    dialogs::Submission::Service { zone, service } => {
                        tasks.push(self.update_zones(zones::Message::AddService { zone, service }));
                    }
                    dialogs::Submission::Port {
                        zone,
                        port,
                        protocol,
                    } => {
                        tasks.push(self.update_zones(zones::Message::AddPort {
                            zone,
                            port,
                            protocol,
                        }));
                    }
                    dialogs::Submission::SourcePort {
                        zone,
                        port,
                        protocol,
                    } => {
                        tasks.push(self.update_zones(zones::Message::AddSourcePort {
                            zone,
                            port,
                            protocol,
                        }));
                    }
                    dialogs::Submission::ForwardPort {
                        zone,
                        port,
                        protocol,
                        to_port,
                        to_addr,
                    } => {
                        tasks.push(self.update_zones(zones::Message::AddForwardPort {
                            zone,
                            port,
                            protocol,
                            to_port,
                            to_addr,
                        }));
                    }
                    dialogs::Submission::Interface { zone, interface } => {
                        tasks.push(
                            self.update_zones(zones::Message::AddInterface { zone, interface }),
                        );
                    }
                    dialogs::Submission::Source { zone, source } => {
                        tasks.push(self.update_zones(zones::Message::AddSource { zone, source }));
                    }
                    dialogs::Submission::Icmp { zone, icmp } => {
                        tasks.push(self.update_zones(zones::Message::AddIcmp { zone, icmp }));
                    }
                    dialogs::Submission::RichRule { zone, rule } => {
                        tasks.push(self.update_zones(zones::Message::AddRichRule { zone, rule }));
                    }
                    dialogs::Submission::IpSet {
                        name,
                        ipset_type,
                        entries,
                    } => {
                        tasks.push(self.update_ipsets(ipsets::Message::Create {
                            name,
                            ipset_type,
                            entries,
                        }));
                    }
                },
                dialogs::Request::CloseDrawer => self.close_context_drawer(),
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

/// Static root and presentation policy for one context page.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ContextPageDescriptor {
    pub(crate) dialog: Option<DialogKind>,
    pub(crate) catalog: Option<ContextCatalog>,
    pub(crate) requires_zone: bool,
    pub(crate) title: ContextTitle,
    pub(crate) footer: ContextFooter,
}

/// Catalog projection loaded whenever a context page opens.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ContextCatalog {
    Services,
    Interfaces,
    IcmpTypes,
}

/// Localized title policy applied by the drawer renderer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ContextTitle {
    None,
    Reconciliation,
    Zone,
    Service,
    Port,
    Interface,
    Source,
    Icmp,
    RichRule,
    IpSet,
}

/// Footer behavior applied by the drawer renderer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ContextFooter {
    None,
    Submit,
    Cancel,
}

impl ContextPage {
    /// Return the single descriptor shared by root routing and drawer rendering.
    pub(crate) const fn descriptor(self) -> ContextPageDescriptor {
        match self {
            Self::About => ContextPageDescriptor {
                dialog: None,
                catalog: None,
                requires_zone: false,
                title: ContextTitle::None,
                footer: ContextFooter::None,
            },
            Self::ReviewReconciliation => ContextPageDescriptor {
                dialog: None,
                catalog: None,
                requires_zone: false,
                title: ContextTitle::Reconciliation,
                footer: ContextFooter::None,
            },
            Self::AddZone => ContextPageDescriptor {
                dialog: Some(DialogKind::Zone),
                catalog: None,
                requires_zone: false,
                title: ContextTitle::Zone,
                footer: ContextFooter::Submit,
            },
            Self::AddService => ContextPageDescriptor {
                dialog: Some(DialogKind::Service),
                catalog: Some(ContextCatalog::Services),
                requires_zone: true,
                title: ContextTitle::Service,
                footer: ContextFooter::Cancel,
            },
            Self::AddPort => ContextPageDescriptor {
                dialog: Some(DialogKind::Port),
                catalog: None,
                requires_zone: true,
                title: ContextTitle::Port,
                footer: ContextFooter::Submit,
            },
            Self::AddInterface => ContextPageDescriptor {
                dialog: Some(DialogKind::Interface),
                catalog: Some(ContextCatalog::Interfaces),
                requires_zone: true,
                title: ContextTitle::Interface,
                footer: ContextFooter::Submit,
            },
            Self::AddSource => ContextPageDescriptor {
                dialog: Some(DialogKind::Source),
                catalog: None,
                requires_zone: true,
                title: ContextTitle::Source,
                footer: ContextFooter::Submit,
            },
            Self::AddIcmp => ContextPageDescriptor {
                dialog: Some(DialogKind::Icmp),
                catalog: Some(ContextCatalog::IcmpTypes),
                requires_zone: true,
                title: ContextTitle::Icmp,
                footer: ContextFooter::Cancel,
            },
            Self::AddRichRule => ContextPageDescriptor {
                dialog: Some(DialogKind::RichRule),
                catalog: None,
                requires_zone: true,
                title: ContextTitle::RichRule,
                footer: ContextFooter::Submit,
            },
            Self::AddIpSet => ContextPageDescriptor {
                dialog: Some(DialogKind::IpSet),
                catalog: None,
                requires_zone: false,
                title: ContextTitle::IpSet,
                footer: ContextFooter::Submit,
            },
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MenuAction {
    About,
    AddZone,
    AddService,
    AddPort,
    AddForwardPort,
    AddSourcePort,
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
            MenuAction::AddService => {
                Message::Zone(zones::Message::View(zones::ZoneViewAction::AddService))
            }
            MenuAction::AddPort => {
                Message::Zone(zones::Message::View(zones::ZoneViewAction::AddPort {
                    kind: dialogs::PortKind::Destination,
                }))
            }
            MenuAction::AddForwardPort => {
                Message::Zone(zones::Message::View(zones::ZoneViewAction::AddPort {
                    kind: dialogs::PortKind::Forward,
                }))
            }
            MenuAction::AddSourcePort => {
                Message::Zone(zones::Message::View(zones::ZoneViewAction::AddPort {
                    kind: dialogs::PortKind::Source,
                }))
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
    fn context_page_descriptors_centralize_dialog_catalog_and_presentation_policy() {
        let cases = [
            (
                ContextPage::About,
                None,
                None,
                false,
                ContextTitle::None,
                ContextFooter::None,
            ),
            (
                ContextPage::ReviewReconciliation,
                None,
                None,
                false,
                ContextTitle::Reconciliation,
                ContextFooter::None,
            ),
            (
                ContextPage::AddZone,
                Some(DialogKind::Zone),
                None,
                false,
                ContextTitle::Zone,
                ContextFooter::Submit,
            ),
            (
                ContextPage::AddService,
                Some(DialogKind::Service),
                Some(ContextCatalog::Services),
                true,
                ContextTitle::Service,
                ContextFooter::Cancel,
            ),
            (
                ContextPage::AddPort,
                Some(DialogKind::Port),
                None,
                true,
                ContextTitle::Port,
                ContextFooter::Submit,
            ),
            (
                ContextPage::AddInterface,
                Some(DialogKind::Interface),
                Some(ContextCatalog::Interfaces),
                true,
                ContextTitle::Interface,
                ContextFooter::Submit,
            ),
            (
                ContextPage::AddSource,
                Some(DialogKind::Source),
                None,
                true,
                ContextTitle::Source,
                ContextFooter::Submit,
            ),
            (
                ContextPage::AddIcmp,
                Some(DialogKind::Icmp),
                Some(ContextCatalog::IcmpTypes),
                true,
                ContextTitle::Icmp,
                ContextFooter::Cancel,
            ),
            (
                ContextPage::AddRichRule,
                Some(DialogKind::RichRule),
                None,
                true,
                ContextTitle::RichRule,
                ContextFooter::Submit,
            ),
            (
                ContextPage::AddIpSet,
                Some(DialogKind::IpSet),
                None,
                false,
                ContextTitle::IpSet,
                ContextFooter::Submit,
            ),
        ];

        for (page, dialog, catalog, requires_zone, title, footer) in cases {
            assert_eq!(
                page.descriptor(),
                ContextPageDescriptor {
                    dialog,
                    catalog,
                    requires_zone,
                    title,
                    footer,
                }
            );
        }
    }

    #[test]
    fn dialog_cancellation_and_pending_mutations_are_routed_safely() {
        let mut state = DialogState::default();
        state.zone.name = "work".into();
        let outcome = dialogs::update(
            &mut state,
            DialogMessage::Cancel(DialogKind::Zone),
            dialog_context(false),
        );
        let mut router = router::Router::new(outcome);
        assert!(matches!(
            router.pop_request(),
            Some(dialogs::Request::CloseDrawer)
        ));
        assert!(router.pop_request().is_none());
        assert!(router.into_effects().is_empty());

        state.zone.name = "work".into();
        let outcome = dialogs::update(
            &mut state,
            DialogMessage::Submit(DialogKind::Zone),
            dialog_context(true),
        );
        assert!(outcome.requests.is_empty());
    }

    #[test]
    fn reconciliation_refresh_requests_are_drained_fifo() {
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
            router.pop_request(),
            Some(reconciliation::Request::RefreshFirewalldStatus)
        ));
        assert!(matches!(
            router.pop_request(),
            Some(reconciliation::Request::RefreshZones)
        ));
        assert!(matches!(
            router.pop_request(),
            Some(reconciliation::Request::RefreshIpSets)
        ));
        assert!(matches!(
            router.pop_request(),
            Some(reconciliation::Request::RefreshCatalogs)
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

    #[test]
    fn reloaded_configuration_plans_the_complete_ordered_refresh() {
        assert_eq!(
            plan_configuration_refresh(
                ConfigurationEvent::Reloaded,
                ConfigurationRefreshContext {
                    selected_zone: Some("public"),
                },
            ),
            ConfigurationRefreshPlan::ReloadEverything
        );
        assert_eq!(
            RELOAD_EVERYTHING_STEPS,
            [
                ReloadEverythingStep::FirewalldStatus,
                ReloadEverythingStep::Zones,
                ReloadEverythingStep::IpSets,
                ReloadEverythingStep::Services,
                ReloadEverythingStep::IcmpTypes,
                ReloadEverythingStep::Interfaces,
            ]
        );
    }

    #[test]
    fn runtime_zone_refresh_only_targets_the_current_reconciliation() {
        assert_eq!(
            plan_configuration_refresh(
                ConfigurationEvent::RuntimeZoneChanged {
                    zone: "public".into(),
                },
                ConfigurationRefreshContext {
                    selected_zone: Some("public"),
                },
            ),
            ConfigurationRefreshPlan::ReloadCurrentReconciliation {
                zone: "public".into(),
            }
        );
        assert_eq!(
            plan_configuration_refresh(
                ConfigurationEvent::RuntimeZoneChanged {
                    zone: "work".into(),
                },
                ConfigurationRefreshContext {
                    selected_zone: Some("public"),
                },
            ),
            ConfigurationRefreshPlan::FinishWithoutReload
        );
    }

    #[test]
    fn permanent_zone_update_only_reloads_current_zone_details() {
        assert_eq!(
            plan_configuration_refresh(
                ConfigurationEvent::PermanentZoneUpdated {
                    zone: "public".into(),
                },
                ConfigurationRefreshContext {
                    selected_zone: Some("public"),
                },
            ),
            ConfigurationRefreshPlan::ReloadCurrentZone {
                zone: "public".into(),
            }
        );
        assert_eq!(
            plan_configuration_refresh(
                ConfigurationEvent::PermanentZoneUpdated {
                    zone: "work".into(),
                },
                ConfigurationRefreshContext {
                    selected_zone: Some("public"),
                },
            ),
            ConfigurationRefreshPlan::FinishWithoutReload
        );
    }

    #[test]
    fn permanent_removal_reloads_the_zone_list() {
        assert_eq!(
            plan_configuration_refresh(
                ConfigurationEvent::PermanentZoneRemoved {
                    zone: "work".into(),
                },
                ConfigurationRefreshContext {
                    selected_zone: Some("public"),
                },
            ),
            ConfigurationRefreshPlan::ReloadZones
        );
    }

    #[test]
    fn permanent_rename_preserves_selection_before_reloading_zones() {
        let plan = plan_configuration_refresh(
            ConfigurationEvent::PermanentZoneRenamed {
                old_zone: "public".into(),
                new_zone: "public2".into(),
            },
            ConfigurationRefreshContext {
                selected_zone: Some("public"),
            },
        );
        assert_eq!(
            plan,
            ConfigurationRefreshPlan::PreserveRenameAndReloadZones {
                old_zone: "public".into(),
                new_zone: "public2".into(),
            }
        );

        let mut navigation = navigation::State::new();
        navigation.set_zones(vec!["public".into()]);
        let zone_id = navigation.zone_id("public").expect("zone is materialized");
        let _ = navigation::update(
            &mut navigation,
            navigation::Message::Select(zone_id),
            navigation::Context,
        );
        navigation.preserve_zone_rename("public", "public2");
        navigation.set_zones(vec!["public2".into()]);
        assert!(matches!(
            navigation.active_item(),
            Some(SidebarItem::Zone { name, .. }) if name == "public2"
        ));
    }
}
