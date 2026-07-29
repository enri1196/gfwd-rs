// SPDX-License-Identifier: MIT

use crate::config::Config;
use crate::core::{BrokerError, ConfigurationEvent, FirewalldStatus};
use crate::fl;
use cosmic::cosmic_config::{self, CosmicConfigEntry};
use cosmic::iced::Subscription;
use cosmic::prelude::*;
use cosmic::widget::{self, Toast, ToastId, about::About, menu, nav_bar};
use dialogs::{DialogKind, DialogMessage, DialogState, localized_validation_error};
use navigation::SidebarItem;
use std::collections::{HashMap, HashSet};
use zones::{ZoneViewAction, ZoneViewState};

const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");
const APP_ICON: &[u8] = include_bytes!("../resources/icons/hicolor/scalable/apps/icon.svg");

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
            Message::Navigation(navigation::Message::ToggleContextPage(context_page)) => {
                let mut tasks = Vec::new();
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
                    // Close the context drawer if the toggled context page is the same.
                    self.core.window.show_context = !self.core.window.show_context;
                } else {
                    // Open the context drawer to display the requested context page.
                    self.context_page = context_page;
                    self.core.window.show_context = true;
                }
                if self.core.window.show_context {
                    self.reset_dialog_for_context(context_page);
                    if requires_zone && self.current_zone_name().is_none() {
                        self.dialogs.operation_error = Some(fl!("error-select-zone-first"));
                    }
                    if context_page == ContextPage::AddInterface {
                        tasks.push(self.update_catalogs(catalogs::Message::LoadInterfaces));
                    } else if context_page == ContextPage::AddService {
                        tasks.push(self.update_catalogs(catalogs::Message::LoadServices));
                    } else if context_page == ContextPage::AddIcmp {
                        tasks.push(self.update_catalogs(catalogs::Message::LoadIcmpTypes));
                    }
                }
                if !tasks.is_empty() {
                    return Task::batch(tasks);
                }
            }

            Message::Dialog(dialog_message) => {
                return self.handle_dialog_message(dialog_message);
            }

            Message::Navigation(navigation::Message::MenuAction(action)) => {
                return self.handle_nav_menu_action(action);
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

            Message::Navigation(navigation::Message::UpdateConfig(config)) => {
                self.config = config;
            }

            Message::Navigation(navigation::Message::LaunchUrl(url)) => {
                match open::that_detached(&url) {
                    Ok(()) => {}
                    Err(err) => {
                        eprintln!("failed to open {url:?}: {err}");
                    }
                }
            }
            Message::Zone(zones::Message::ListLoaded(result)) => {
                let mut tasks = Vec::new();
                match result {
                    Ok(zones) => {
                        self.navigation.set_zones(zones);
                        tasks.push(zones::effects::start_default_zone_load(self));
                        tasks.push(zones::effects::start_active_zones_load(self));
                    }
                    Err(error) => {
                        eprintln!("failed to load zones: {error}");
                        self.navigation.set_error(error.to_string());
                        self.zones = ZoneViewState::Error {
                            zone: "zones".to_string(),
                            message: error.to_string(),
                        };
                    }
                }
                let task = match self.navigation.active_item() {
                    Some(SidebarItem::Zone { name, .. }) => {
                        zones::effects::start_zone_load(self, name.clone())
                    }
                    _ => {
                        if !matches!(self.zones, ZoneViewState::Error { .. }) {
                            self.zones = ZoneViewState::Empty;
                            self.reconciliation.set_unavailable(None);
                        }
                        self.finish_configuration_refresh()
                    }
                };

                tasks.push(self.update_title());
                tasks.push(task);
                return Task::batch(tasks);
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
            Message::Zone(zones::Message::DefaultLoaded(result)) => match result {
                Ok(zone) => self.navigation.set_default_zone(Some(zone)),
                Err(error) => {
                    eprintln!("failed to load default zone: {error}");
                    self.navigation.set_default_zone(None);
                }
            },
            Message::Zone(zones::Message::ActiveLoaded(result)) => match result {
                Ok(active_zones) => self.navigation.set_active_zones(active_zones),
                Err(error) => {
                    eprintln!("failed to load active zones: {error}");
                    self.navigation.set_active_zones(HashSet::new());
                }
            },
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
        self.handle_nav_select(id)
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
            match request {
                reconciliation::Request::OpenReview => {
                    self.open_context_page(ContextPage::ReviewReconciliation);
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
                    tasks.push(zones::effects::start_firewalld_status_load(self));
                }
                reconciliation::Request::RefreshZones => {
                    tasks.push(zones::effects::start_zones_load(self));
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

    fn handle_nav_select(&mut self, id: nav_bar::Id) -> Task<cosmic::Action<Message>> {
        self.navigation.activate(id);

        let task = match self.navigation.active_item() {
            Some(SidebarItem::Zone { name, .. }) => {
                zones::effects::start_zone_load(self, name.clone())
            }
            Some(SidebarItem::IpSets) => self.update_ipsets(ipsets::Message::LoadList),
            _ => {
                self.zones = ZoneViewState::Empty;
                Task::none()
            }
        };

        Task::batch(vec![self.update_title(), task])
    }

    fn handle_nav_menu_action(&mut self, action: NavMenuAction) -> Task<cosmic::Action<Message>> {
        match action {
            NavMenuAction::Open(id) => self.handle_nav_select(id),
            NavMenuAction::Activate(id) => {
                if self.navigation.zone_name_for_id(id).is_none() {
                    return Task::none();
                }
                let task = self.handle_nav_select(id);
                self.context_page = ContextPage::AddInterface;
                self.core.window.show_context = true;
                self.reset_dialog_for_context(ContextPage::AddInterface);
                Task::batch(vec![
                    task,
                    self.update_catalogs(catalogs::Message::LoadInterfaces),
                ])
            }
            NavMenuAction::SetDefault(id) => {
                let Some(zone_name) = self.navigation.zone_name_for_id(id) else {
                    return Task::none();
                };
                zones::effects::start_default_zone_set(self, zone_name)
            }
            NavMenuAction::Delete(id) => {
                let Some(zone_name) = self.navigation.zone_name_for_id(id) else {
                    return Task::none();
                };
                self.operations.confirmation = Some(Confirmation::DeleteZone(zone_name));
                Task::none()
            }
        }
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
            match request {
                dialogs::Request::Submit(submission) => {
                    let task = match submission {
                        dialogs::Submission::Zone {
                            name,
                            description,
                            target,
                        } => zones::effects::start_zone_create(self, name, description, target),
                        dialogs::Submission::Service { zone, service } => {
                            zones::effects::start_service_add(self, zone, service)
                        }
                        dialogs::Submission::Port {
                            zone,
                            port,
                            protocol,
                        } => zones::effects::start_port_add(self, zone, port, protocol),
                        dialogs::Submission::SourcePort {
                            zone,
                            port,
                            protocol,
                        } => zones::effects::start_source_port_add(self, zone, port, protocol),
                        dialogs::Submission::ForwardPort {
                            zone,
                            port,
                            protocol,
                            to_port,
                            to_addr,
                        } => zones::effects::start_forward_port_add(
                            self, zone, port, protocol, to_port, to_addr,
                        ),
                        dialogs::Submission::Interface { zone, interface } => {
                            zones::effects::start_interface_add(self, zone, interface)
                        }
                        dialogs::Submission::Source { zone, source } => {
                            zones::effects::start_source_add(self, zone, source)
                        }
                        dialogs::Submission::Icmp { zone, icmp } => {
                            zones::effects::start_icmp_add(self, zone, icmp)
                        }
                        dialogs::Submission::RichRule { zone, rule } => {
                            zones::effects::start_rich_rule_add(self, zone, rule)
                        }
                        dialogs::Submission::IpSet {
                            name,
                            ipset_type,
                            entries,
                        } => self.start_ipset_create(name, ipset_type, entries),
                    };
                    tasks.push(task);
                }
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
pub enum NavMenuAction {
    Open(nav_bar::Id),
    Activate(nav_bar::Id),
    SetDefault(nav_bar::Id),
    Delete(nav_bar::Id),
}

impl menu::action::MenuAction for NavMenuAction {
    type Message = cosmic::Action<Message>;

    fn message(&self) -> Self::Message {
        cosmic::Action::App(Message::Navigation(navigation::Message::MenuAction(*self)))
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
