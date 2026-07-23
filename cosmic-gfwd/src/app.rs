// SPDX-License-Identifier: MIT

use crate::config::Config;
use crate::core::{
    BrokerError, ConfigurationEvent, ConfigurationRefreshCoordinator, FirewalldStatus, FwdBroker,
    RefreshRequest, ZoneReconciliationData, ZoneReconciliationState, validate_interface_name,
    validate_ipset_entry,
};
use crate::fl;
use crate::models::{IcmpTypeInfo, IpSetDetails, ZoneDetails, ZoneTarget};
use crate::ui::{
    DialogKind, DialogMessage, DialogState, IpSetViewAction, IpSetViewState, Sidebar, SidebarItem,
    ZoneViewAction, ZoneViewState, drawer_cancel_footer, drawer_footer_with_submit,
    drawer_with_error, icmp_drawer, interface_drawer, ipset_drawer, localized_validation_error,
    port_drawer, reconciliation_drawer, rich_rule_drawer, service_drawer, source_drawer,
    target_from_index, view_ipset_content, view_zone_content,
};
use cosmic::app::context_drawer;
use cosmic::cosmic_config::{self, CosmicConfigEntry};
use cosmic::iced::alignment::{Horizontal, Vertical};
use cosmic::iced::{Length, Subscription};
use cosmic::prelude::*;
use cosmic::widget::{self, Toast, ToastId, Toasts, about::About, menu, nav_bar};
use futures_util::{StreamExt, stream::BoxStream};
use slotmap::Key;
use std::collections::{HashMap, HashSet};

const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");
const APP_ICON: &[u8] = include_bytes!("../resources/icons/hicolor/scalable/apps/icon.svg");

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
    sidebar: Sidebar,
    /// State for the current zone detail view.
    zone_view: ZoneViewState,
    /// Runtime/permanent comparison loaded independently from permanent details.
    zone_reconciliation: ZoneReconciliationState,
    /// Pure selected-zone request, coalescing, and watcher state.
    configuration_coordinator: ConfigurationRefreshCoordinator,
    /// State for the IP set view.
    ipset_view: IpSetViewState,
    /// Stores form state for context drawer dialogs.
    dialogs: DialogState,
    /// Available network interfaces for the interface dialog.
    interface_options: Vec<String>,
    /// Whether interface discovery is in progress.
    interface_loading: bool,
    /// Error message when interface discovery fails.
    interface_error: Option<String>,
    /// Permanent firewalld service catalog.
    service_options: Vec<String>,
    /// Whether the service catalog is being loaded.
    service_loading: bool,
    /// Error returned while loading the service catalog.
    service_error: Option<String>,
    /// Configured ICMP types available to block.
    icmp_options: Vec<IcmpTypeInfo>,
    /// Whether the ICMP catalog is being loaded.
    icmp_loading: bool,
    /// Error returned while loading the ICMP catalog.
    icmp_error: Option<String>,
    /// User-visible notifications for completed operations.
    toasts: Toasts<Message>,
    /// Name of the mutation currently in flight.
    pending_operation: Option<String>,
    /// Destructive operation awaiting explicit confirmation.
    confirmation: Option<Confirmation>,
    /// Current activation state of the firewalld systemd unit.
    firewalld_status: FirewalldStatus,
    /// Permanent configuration has changed since the last explicit runtime reload.
    runtime_reload_needed: bool,
    /// Key bindings for the application's menu bar.
    key_binds: HashMap<menu::KeyBind, MenuAction>,
    /// Configuration data that persists between application runs.
    config: Config,
}

/// Messages emitted by the application and its widgets.
#[derive(Debug, Clone)]
pub enum Message {
    LaunchUrl(String),
    ToggleContextPage(ContextPage),
    Dialog(DialogMessage),
    NavMenuAction(NavMenuAction),
    ZoneAction(ZoneViewAction),
    IpSetAction(IpSetViewAction),
    UpdateConfig(Config),
    ZonesLoaded(Result<Vec<String>, BrokerError>),
    ZoneDetailsLoaded {
        zone_name: String,
        result: Result<ZoneDetails, BrokerError>,
    },
    ZoneReconciliationLoaded {
        zone_name: String,
        generation: u64,
        result: Box<Result<ZoneReconciliationData, BrokerError>>,
    },
    ConfigurationEvent(Result<ConfigurationEvent, BrokerError>),
    DefaultZoneLoaded(Result<String, BrokerError>),
    ActiveZonesLoaded(Result<HashSet<String>, BrokerError>),
    InterfacesLoaded(Result<Vec<String>, BrokerError>),
    ServicesLoaded(Result<Vec<String>, BrokerError>),
    IcmpTypesLoaded(Result<Vec<IcmpTypeInfo>, BrokerError>),
    DefaultZoneSet(Result<(), BrokerError>),
    ZoneCreated {
        zone_name: String,
        result: Result<(), BrokerError>,
    },
    ZoneDeleted {
        zone_name: String,
        result: Result<(), BrokerError>,
    },
    ZoneItemAdded {
        zone_name: String,
        result: Result<(), BrokerError>,
    },
    ZoneItemRemoved {
        zone_name: String,
        result: Result<(), BrokerError>,
    },
    IpSetsLoaded(Result<Vec<String>, BrokerError>),
    IpSetEntryRemoved {
        ipset_name: String,
        result: Result<(), BrokerError>,
    },
    IpSetDetailsLoaded {
        ipset_name: String,
        result: Result<IpSetDetails, BrokerError>,
    },
    IpSetEntryAdded {
        ipset_name: String,
        result: Result<(), BrokerError>,
    },
    IpSetCreated {
        ipset_name: String,
        result: Result<(), BrokerError>,
    },
    IpSetDeleted {
        ipset_name: String,
        result: Result<(), BrokerError>,
    },
    FirewalldStatusLoaded(Result<FirewalldStatus, BrokerError>),
    FirewalldControlFinished {
        apply_permanent: bool,
        result: Result<(), BrokerError>,
    },
    RuntimeConfigurationPersisted(Result<(), BrokerError>),
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
        let sidebar = Sidebar::new();

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
            sidebar,
            zone_view: ZoneViewState::Empty,
            zone_reconciliation: ZoneReconciliationState::default(),
            configuration_coordinator: ConfigurationRefreshCoordinator::default(),
            ipset_view: IpSetViewState::default(),
            dialogs: DialogState::default(),
            interface_options: Vec::new(),
            interface_loading: false,
            interface_error: None,
            service_options: Vec::new(),
            service_loading: false,
            service_error: None,
            icmp_options: Vec::new(),
            icmp_loading: false,
            icmp_error: None,
            toasts: Toasts::new(Message::DismissToast),
            pending_operation: None,
            confirmation: None,
            firewalld_status: FirewalldStatus::Loading,
            runtime_reload_needed: false,
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

        let command = Task::batch(vec![
            app.update_title(),
            app.start_zones_load(),
            app.start_firewalld_status_load(),
        ]);

        (app, command)
    }

    /// Elements to pack at the start of the header bar.
    fn header_start(&self) -> Vec<Element<'_, Self::Message>> {
        let menu_bar = menu::bar(vec![
            menu::Tree::with_children(
                menu::root(fl!("menu-zones")).apply(Element::from),
                menu::items(
                    &self.key_binds,
                    vec![menu::Item::Button(
                        fl!("action-add-zone"),
                        None,
                        MenuAction::AddZone,
                    )],
                ),
            ),
            menu::Tree::with_children(
                menu::root(fl!("menu-rules")).apply(Element::from),
                menu::items(
                    &self.key_binds,
                    vec![
                        menu::Item::Button(fl!("action-add-port"), None, MenuAction::AddPort),
                        menu::Item::Button(
                            fl!("action-add-rich-rule"),
                            None,
                            MenuAction::AddRichRule,
                        ),
                        menu::Item::Button(fl!("action-add-icmp"), None, MenuAction::AddIcmp),
                        menu::Item::Button(fl!("action-add-source"), None, MenuAction::AddSource),
                        menu::Item::Button(
                            fl!("action-add-interface"),
                            None,
                            MenuAction::AddInterface,
                        ),
                    ],
                ),
            ),
            menu::Tree::with_children(
                menu::root(fl!("menu-objects")).apply(Element::from),
                menu::items(
                    &self.key_binds,
                    vec![menu::Item::Button(
                        fl!("action-add-ipset"),
                        None,
                        MenuAction::AddIpSet,
                    )],
                ),
            ),
            menu::Tree::with_children(
                menu::root(fl!("menu-help")).apply(Element::from),
                menu::items(
                    &self.key_binds,
                    vec![menu::Item::Button(fl!("about"), None, MenuAction::About)],
                ),
            ),
        ]);

        vec![menu_bar.into()]
    }

    /// Enables the COSMIC application to create a nav bar with this model.
    fn nav_model(&self) -> Option<&nav_bar::Model> {
        Some(self.sidebar.nav_model())
    }

    /// The context menu to display for the given nav bar item ID.
    fn nav_context_menu(
        &self,
        id: nav_bar::Id,
    ) -> Option<Vec<menu::Tree<cosmic::Action<Self::Message>>>> {
        let context_id = if id.is_null() {
            self.sidebar.nav_model().active()
        } else {
            id
        };

        let Some(item) = self.sidebar.item_for_id(context_id) else {
            return Some(Vec::new());
        };

        match item {
            SidebarItem::Zone { .. } => {
                let key_binds = HashMap::new();
                Some(menu::items(
                    &key_binds,
                    vec![
                        menu::Item::Button(
                            fl!("context-open-zone"),
                            None,
                            NavMenuAction::Open(context_id),
                        ),
                        menu::Item::Button(
                            fl!("context-activate-zone"),
                            None,
                            NavMenuAction::Activate(context_id),
                        ),
                        menu::Item::Button(
                            fl!("context-set-default-zone"),
                            None,
                            NavMenuAction::SetDefault(context_id),
                        ),
                        menu::Item::Button(
                            fl!("context-delete-zone"),
                            None,
                            NavMenuAction::Delete(context_id),
                        ),
                    ],
                ))
            }
            _ => Some(Vec::new()),
        }
    }

    /// Display a context drawer if the context page is requested.
    fn context_drawer(&self) -> Option<context_drawer::ContextDrawer<'_, Self::Message>> {
        if !self.core.window.show_context {
            return None;
        }

        let interface_error = self
            .dialogs
            .interface
            .error
            .as_deref()
            .or(self.interface_error.as_deref());
        let can_submit_interface = self.dialogs.interface.error.is_none()
            && !self.dialogs.interface.interface.trim().is_empty();
        let can_submit = !self.mutation_pending();
        let error = self.dialogs.operation_error.as_deref();
        let enabled_services = match &self.zone_view {
            ZoneViewState::Ready(details) => details.services.as_slice(),
            _ => &[],
        };
        let blocked_icmp = match &self.zone_view {
            ZoneViewState::Ready(details) => details.icmp_blocks.as_slice(),
            _ => &[],
        };

        Some(match self.context_page {
            ContextPage::About => context_drawer::about(
                &self.about,
                |url| Message::LaunchUrl(url.to_string()),
                Message::ToggleContextPage(ContextPage::About),
            ),
            ContextPage::ReviewReconciliation => context_drawer::context_drawer(
                reconciliation_drawer(
                    &self.zone_reconciliation,
                    self.mutation_pending(),
                    self.dialogs.operation_error.as_deref(),
                    self.configuration_coordinator.watch_warning(),
                    Message::ZoneAction,
                ),
                Message::ToggleContextPage(ContextPage::ReviewReconciliation),
            )
            .title(fl!("reconciliation-review-title")),
            ContextPage::AddZone => context_drawer::context_drawer(
                drawer_with_error(
                    crate::ui::dialog_drawers::zone_drawer(&self.dialogs.zone),
                    error,
                ),
                DialogMessage::Cancel(DialogKind::Zone),
            )
            .title(fl!("drawer-title-zone"))
            .footer(drawer_footer_with_submit(
                DialogKind::Zone,
                can_submit && !self.dialogs.zone.name.trim().is_empty(),
            ))
            .map(Message::Dialog),
            ContextPage::AddService => context_drawer::context_drawer(
                drawer_with_error(
                    service_drawer(
                        &self.dialogs.service,
                        &self.service_options,
                        enabled_services,
                        self.service_loading,
                        self.service_error.as_deref(),
                    ),
                    error,
                ),
                DialogMessage::Cancel(DialogKind::Service),
            )
            .title(fl!("dialog-service-title"))
            .footer(drawer_cancel_footer(DialogKind::Service))
            .map(Message::Dialog),
            ContextPage::AddPort => context_drawer::context_drawer(
                drawer_with_error(port_drawer(&self.dialogs.port), error),
                DialogMessage::Cancel(DialogKind::Port),
            )
            .title(fl!("drawer-title-port"))
            .footer(drawer_footer_with_submit(
                DialogKind::Port,
                can_submit && self.dialogs.port.is_valid(),
            ))
            .map(Message::Dialog),
            ContextPage::AddInterface => context_drawer::context_drawer(
                drawer_with_error(
                    interface_drawer(
                        &self.dialogs.interface,
                        &self.interface_options,
                        self.interface_loading,
                        interface_error,
                    ),
                    error,
                ),
                DialogMessage::Cancel(DialogKind::Interface),
            )
            .title(fl!("drawer-title-interface"))
            .footer(drawer_footer_with_submit(
                DialogKind::Interface,
                can_submit && can_submit_interface,
            ))
            .map(Message::Dialog),
            ContextPage::AddSource => context_drawer::context_drawer(
                drawer_with_error(source_drawer(&self.dialogs.source), error),
                DialogMessage::Cancel(DialogKind::Source),
            )
            .title(fl!("drawer-title-source"))
            .footer(drawer_footer_with_submit(
                DialogKind::Source,
                can_submit && self.dialogs.source.is_valid(),
            ))
            .map(Message::Dialog),
            ContextPage::AddIcmp => context_drawer::context_drawer(
                drawer_with_error(
                    icmp_drawer(
                        &self.dialogs.icmp,
                        &self.icmp_options,
                        blocked_icmp,
                        self.icmp_loading,
                        self.icmp_error.as_deref(),
                    ),
                    error,
                ),
                DialogMessage::Cancel(DialogKind::Icmp),
            )
            .title(fl!("drawer-title-icmp"))
            .footer(drawer_cancel_footer(DialogKind::Icmp))
            .map(Message::Dialog),
            ContextPage::AddRichRule => context_drawer::context_drawer(
                drawer_with_error(rich_rule_drawer(&self.dialogs.rich_rule), error),
                DialogMessage::Cancel(DialogKind::RichRule),
            )
            .title(fl!("drawer-title-rich-rule"))
            .footer(drawer_footer_with_submit(
                DialogKind::RichRule,
                can_submit && self.dialogs.rich_rule.generated_rule().is_ok(),
            ))
            .map(Message::Dialog),
            ContextPage::AddIpSet => context_drawer::context_drawer(
                drawer_with_error(ipset_drawer(&self.dialogs.ipset), error),
                DialogMessage::Cancel(DialogKind::IpSet),
            )
            .title(fl!("drawer-title-ipset"))
            .footer(drawer_footer_with_submit(
                DialogKind::IpSet,
                can_submit && self.dialogs.ipset.is_valid(),
            ))
            .map(Message::Dialog),
        })
    }

    fn dialog(&self) -> Option<Element<'_, Self::Message>> {
        let confirmation = self.confirmation.as_ref()?;
        let (title, body, confirm_label) = match confirmation {
            Confirmation::DeleteZone(zone) => (
                fl!("confirm-delete-zone-title"),
                fl!("confirm-delete-zone-body", zone = zone),
                fl!("confirm-delete"),
            ),
            Confirmation::DeleteIpSet(ipset) => (
                fl!("confirm-delete-ipset-title"),
                fl!("confirm-delete-ipset-body", ipset = ipset),
                fl!("confirm-delete"),
            ),
            Confirmation::StopFirewalld => (
                fl!("confirm-stop-firewalld-title"),
                fl!("confirm-stop-firewalld-body"),
                fl!("firewalld-stop"),
            ),
            Confirmation::ApplyPermanentConfiguration => (
                fl!("confirm-apply-permanent-title"),
                fl!("confirm-apply-permanent-body"),
                fl!("confirm-apply-permanent-action"),
            ),
            Confirmation::SaveRuntimeConfiguration => (
                fl!("confirm-save-runtime-title"),
                fl!("confirm-save-runtime-body"),
                fl!("confirm-save-runtime-action"),
            ),
        };

        Some(
            widget::dialog()
                .title(title)
                .body(body)
                .primary_action(
                    widget::button::destructive(confirm_label)
                        .on_press(Message::ConfirmDestructive),
                )
                .secondary_action(
                    widget::button::text(fl!("dialog-cancel"))
                        .on_press(Message::CancelConfirmation),
                )
                .into(),
        )
    }

    fn footer(&self) -> Option<Element<'_, Self::Message>> {
        self.pending_operation.as_ref().map(|operation| {
            widget::container(widget::text::caption(fl!(
                "operation-pending",
                operation = operation
            )))
            .padding(cosmic::theme::spacing().space_xs)
            .width(Length::Fill)
            .into()
        })
    }

    /// Describes the interface based on the current state of the application model.
    ///
    /// Application events will be processed through the view. Any messages emitted by
    /// events received by widgets will be passed to the update method.
    fn view(&self) -> Element<'_, Self::Message> {
        let space_m = cosmic::theme::spacing().space_m;
        let content: Element<_> = match self.sidebar.active_item() {
            Some(SidebarItem::IpSets) => view_ipset_content(
                &self.ipset_view,
                self.mutation_pending(),
                Message::IpSetAction,
            ),
            _ => view_zone_content(
                &self.zone_view,
                &self.firewalld_status,
                &self.zone_reconciliation,
                self.configuration_coordinator.watch_warning(),
                self.mutation_pending(),
                Message::ZoneAction,
            ),
        };

        widget::toaster(
            &self.toasts,
            widget::container(content)
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(space_m)
                .align_x(Horizontal::Center)
                .align_y(Vertical::Center),
        )
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

                    Message::UpdateConfig(update.config)
                }),
        ];
        let selected_zone = self.current_zone_name();
        subscriptions.push(Subscription::run_with_id(
            ("firewalld-configuration-events", selected_zone.clone()),
            configuration_event_messages(selected_zone),
        ));

        Subscription::batch(subscriptions)
    }

    /// Handles messages emitted by the application and its widgets.
    ///
    /// Tasks may be returned for asynchronous execution of code in the background
    /// on the application's async runtime.
    fn update(&mut self, message: Self::Message) -> Task<cosmic::Action<Self::Message>> {
        match message {
            Message::ToggleContextPage(context_page) => {
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
                        tasks.push(self.start_interfaces_load());
                    } else if context_page == ContextPage::AddService {
                        tasks.push(self.start_services_load());
                    } else if context_page == ContextPage::AddIcmp {
                        tasks.push(self.start_icmp_types_load());
                    }
                }
                if !tasks.is_empty() {
                    return Task::batch(tasks);
                }
            }

            Message::Dialog(dialog_message) => {
                return self.handle_dialog_message(dialog_message);
            }

            Message::NavMenuAction(action) => {
                return self.handle_nav_menu_action(action);
            }

            Message::ZoneAction(action) => {
                return self.handle_zone_action(action);
            }

            Message::IpSetAction(action) => {
                return self.handle_ipset_action(action);
            }
            Message::DismissToast(id) => {
                self.toasts.remove(id);
            }
            Message::CancelConfirmation => {
                self.confirmation = None;
            }
            Message::ConfirmDestructive => {
                let Some(confirmation) = self.confirmation.take() else {
                    return Task::none();
                };
                return match confirmation {
                    Confirmation::DeleteZone(zone_name) => self.start_zone_delete(zone_name),
                    Confirmation::DeleteIpSet(ipset_name) => self.start_ipset_delete(ipset_name),
                    Confirmation::StopFirewalld => self.start_firewalld_control(false),
                    Confirmation::ApplyPermanentConfiguration => self.start_permanent_apply(),
                    Confirmation::SaveRuntimeConfiguration => {
                        self.start_runtime_configuration_persist()
                    }
                };
            }
            Message::ConfigurationEvent(result) => match result {
                Ok(event) => {
                    self.configuration_coordinator.watcher_recovered();
                    return self.schedule_configuration_refresh(event);
                }
                Err(error) => {
                    self.configuration_coordinator
                        .watcher_failed(error.to_string());
                }
            },
            Message::FirewalldStatusLoaded(result) => {
                self.firewalld_status = match result {
                    Ok(status) => status,
                    Err(error) => FirewalldStatus::Error(error.to_string()),
                };
                if self.firewalld_status == FirewalldStatus::Active
                    && !self.configuration_coordinator.is_refreshing()
                {
                    if let ZoneViewState::Ready(details) = &self.zone_view {
                        return self.start_zone_reconciliation(details.name.clone());
                    }
                } else {
                    self.zone_reconciliation = ZoneReconciliationState::Unavailable {
                        zone: self.current_zone_name(),
                    };
                }
            }
            Message::FirewalldControlFinished {
                apply_permanent,
                result,
            } => {
                if result.is_ok() && apply_permanent {
                    self.runtime_reload_needed = false;
                }
                let mut tasks = vec![
                    self.finish_mutation(&result),
                    self.start_firewalld_status_load(),
                ];
                if result.is_ok() && apply_permanent {
                    tasks.push(self.start_zones_load());
                }
                return Task::batch(tasks);
            }
            Message::RuntimeConfigurationPersisted(result) => {
                let mut tasks = vec![self.finish_mutation(&result)];
                if result.is_ok() {
                    tasks.extend([
                        self.start_firewalld_status_load(),
                        self.start_zones_load(),
                        self.start_ipsets_load(),
                        self.start_services_load(),
                        self.start_icmp_types_load(),
                        self.start_interfaces_load(),
                    ]);
                }
                return Task::batch(tasks);
            }

            Message::UpdateConfig(config) => {
                self.config = config;
            }

            Message::LaunchUrl(url) => match open::that_detached(&url) {
                Ok(()) => {}
                Err(err) => {
                    eprintln!("failed to open {url:?}: {err}");
                }
            },
            Message::ZonesLoaded(result) => {
                let mut tasks = Vec::new();
                match result {
                    Ok(zones) => {
                        self.sidebar.set_zones(zones);
                        tasks.push(self.start_default_zone_load());
                        tasks.push(self.start_active_zones_load());
                    }
                    Err(error) => {
                        eprintln!("failed to load zones: {error}");
                        self.sidebar.set_error(error.to_string());
                        self.zone_view = ZoneViewState::Error {
                            zone: "zones".to_string(),
                            message: error.to_string(),
                        };
                    }
                }
                let task = match self.sidebar.active_item() {
                    Some(SidebarItem::Zone { name, .. }) => self.start_zone_load(name.clone()),
                    _ => {
                        if !matches!(self.zone_view, ZoneViewState::Error { .. }) {
                            self.zone_view = ZoneViewState::Empty;
                            self.zone_reconciliation =
                                ZoneReconciliationState::Unavailable { zone: None };
                        }
                        self.finish_configuration_refresh()
                    }
                };

                tasks.push(self.update_title());
                tasks.push(task);
                return Task::batch(tasks);
            }
            Message::ZoneDetailsLoaded { zone_name, result } => {
                let is_active = matches!(
                    self.sidebar.active_item(),
                    Some(SidebarItem::Zone { name, .. }) if name == &zone_name
                );
                if !is_active {
                    return Task::none();
                }

                match result {
                    Ok(details) => {
                        self.zone_view = ZoneViewState::Ready(Box::new(details));
                        if self.firewalld_status == FirewalldStatus::Active {
                            return self.start_zone_reconciliation(zone_name);
                        }
                        self.zone_reconciliation = ZoneReconciliationState::Unavailable {
                            zone: Some(zone_name),
                        };
                        return self.finish_configuration_refresh();
                    }
                    Err(error) => {
                        self.zone_view = ZoneViewState::Error {
                            zone: zone_name.clone(),
                            message: error.to_string(),
                        };
                        self.zone_reconciliation = ZoneReconciliationState::Unavailable {
                            zone: Some(zone_name),
                        };
                        return self.finish_configuration_refresh();
                    }
                }
            }
            Message::ZoneReconciliationLoaded {
                zone_name,
                generation,
                result,
            } => {
                let is_current = self.configuration_coordinator.accepts(generation)
                    && matches!(
                        self.sidebar.active_item(),
                        Some(SidebarItem::Zone { name, .. }) if name == &zone_name
                    )
                    && matches!(
                        &self.zone_view,
                        ZoneViewState::Ready(details) if details.name == zone_name
                    );
                if !is_current {
                    return Task::none();
                }

                self.zone_reconciliation = match *result {
                    Ok(data) => ZoneReconciliationState::from_data(zone_name, data),
                    Err(error) => ZoneReconciliationState::Error {
                        zone: zone_name,
                        message: error.to_string(),
                    },
                };
                return self.finish_configuration_refresh();
            }
            Message::DefaultZoneLoaded(result) => match result {
                Ok(zone) => self.sidebar.set_default_zone(Some(zone)),
                Err(error) => {
                    eprintln!("failed to load default zone: {error}");
                    self.sidebar.set_default_zone(None);
                }
            },
            Message::ActiveZonesLoaded(result) => match result {
                Ok(active_zones) => self.sidebar.set_active_zones(active_zones),
                Err(error) => {
                    eprintln!("failed to load active zones: {error}");
                    self.sidebar.set_active_zones(HashSet::new());
                }
            },
            Message::InterfacesLoaded(result) => {
                self.interface_loading = false;
                match result {
                    Ok(interfaces) => {
                        self.interface_options = interfaces;
                        self.interface_error = None;
                        if !self.interface_options.is_empty()
                            && !self
                                .interface_options
                                .iter()
                                .any(|iface| iface == &self.dialogs.interface.interface)
                        {
                            self.dialogs.interface.interface.clear();
                            self.dialogs.interface.error = None;
                        }
                    }
                    Err(error) => {
                        self.interface_options.clear();
                        self.interface_error = Some(error.to_string());
                    }
                }
            }
            Message::ServicesLoaded(result) => {
                self.service_loading = false;
                match result {
                    Ok(services) => {
                        self.service_options = services;
                        self.service_error = None;
                    }
                    Err(error) => {
                        self.service_options.clear();
                        self.service_error = Some(error.to_string());
                    }
                }
            }
            Message::IcmpTypesLoaded(result) => {
                self.icmp_loading = false;
                match result {
                    Ok(types) => {
                        self.icmp_options = types;
                        self.icmp_error = None;
                    }
                    Err(error) => {
                        self.icmp_options.clear();
                        self.icmp_error = Some(error.to_string());
                    }
                }
            }
            Message::DefaultZoneSet(result) => match result {
                Ok(()) => {
                    return Task::batch(vec![
                        self.finish_mutation(&result),
                        self.start_default_zone_load(),
                    ]);
                }
                Err(error) => {
                    return self.finish_mutation(&Err(error));
                }
            },
            Message::ZoneCreated { zone_name, result } => match result {
                Ok(()) => {
                    self.runtime_reload_needed = true;
                    self.dialogs.reset(DialogKind::Zone);
                    self.close_context_drawer();
                    return Task::batch(vec![
                        self.finish_mutation(&result),
                        self.start_zones_load(),
                    ]);
                }
                Err(error) => {
                    let _ = zone_name;
                    return self.finish_mutation(&Err(error));
                }
            },
            Message::ZoneDeleted { zone_name, result } => match result {
                Ok(()) => {
                    self.runtime_reload_needed = true;
                    if matches!(
                        self.sidebar.active_item(),
                        Some(SidebarItem::Zone { name, .. }) if name == &zone_name
                    ) {
                        self.zone_view = ZoneViewState::Empty;
                        self.zone_reconciliation =
                            ZoneReconciliationState::Unavailable { zone: None };
                    }
                    return Task::batch(vec![
                        self.finish_mutation(&result),
                        self.start_zones_load(),
                    ]);
                }
                Err(error) => {
                    let _ = zone_name;
                    return self.finish_mutation(&Err(error));
                }
            },
            Message::ZoneItemAdded { zone_name, result }
            | Message::ZoneItemRemoved { zone_name, result } => match result {
                Ok(()) => {
                    self.runtime_reload_needed = true;
                    if self.core.window.show_context
                        && let Some(kind) = dialog_kind_for_page(self.context_page)
                    {
                        self.dialogs.reset(kind);
                        self.close_context_drawer();
                    }
                    let is_active = matches!(
                        self.sidebar.active_item(),
                        Some(SidebarItem::Zone { name, .. }) if name == &zone_name
                    );
                    if is_active {
                        return Task::batch(vec![
                            self.finish_mutation(&result),
                            self.start_zone_load(zone_name),
                        ]);
                    }
                    return self.finish_mutation(&result);
                }
                Err(error) => {
                    let _ = zone_name;
                    return self.finish_mutation(&Err(error));
                }
            },
            Message::IpSetsLoaded(result) => {
                self.ipset_view.list_loading = false;
                match result {
                    Ok(ipsets) => {
                        self.ipset_view.ipsets = ipsets;
                        let selected = self.ipset_view.selected.clone();
                        if let Some(name) = selected {
                            if self.ipset_view.ipsets.iter().any(|item| item == &name) {
                                if self.ipset_view.details.is_none() {
                                    return self.start_ipset_details_load(name);
                                }
                            } else {
                                self.ipset_view.selected = None;
                                self.ipset_view.details = None;
                            }
                        }
                    }
                    Err(error) => {
                        self.ipset_view.ipsets.clear();
                        self.ipset_view.entry_error = Some(error.to_string());
                        self.ipset_view.details = None;
                    }
                }
            }
            Message::IpSetDetailsLoaded { ipset_name, result } => {
                let is_active = self.ipset_view.selected.as_deref() == Some(ipset_name.as_str());
                if !is_active {
                    return Task::none();
                }

                self.ipset_view.details_loading = false;
                match result {
                    Ok(details) => {
                        self.ipset_view.details = Some(details);
                        self.ipset_view.entry_error = None;
                    }
                    Err(error) => {
                        self.ipset_view.details = None;
                        self.ipset_view.entry_error = Some(error.to_string());
                    }
                }
            }
            Message::IpSetEntryAdded { ipset_name, result } => match result {
                Ok(()) => {
                    self.runtime_reload_needed = true;
                    self.ipset_view.entry_input.clear();
                    self.ipset_view.entry_error = None;
                    return Task::batch(vec![
                        self.finish_mutation(&result),
                        self.start_ipset_details_load(ipset_name),
                    ]);
                }
                Err(error) => {
                    self.ipset_view.entry_error = Some(error.to_string());
                    return self.finish_mutation(&Err(error));
                }
            },
            Message::IpSetEntryRemoved { ipset_name, result } => match result {
                Ok(()) => {
                    self.runtime_reload_needed = true;
                    self.ipset_view.entry_error = None;
                    return Task::batch(vec![
                        self.finish_mutation(&result),
                        self.start_ipset_details_load(ipset_name),
                    ]);
                }
                Err(error) => {
                    self.ipset_view.entry_error = Some(error.to_string());
                    return self.finish_mutation(&Err(error));
                }
            },
            Message::IpSetCreated { ipset_name, result } => match result {
                Ok(()) => {
                    self.runtime_reload_needed = true;
                    self.dialogs.reset(DialogKind::IpSet);
                    self.close_context_drawer();
                    self.ipset_view.selected = Some(ipset_name.clone());
                    self.ipset_view.entry_input.clear();
                    self.ipset_view.entry_error = None;
                    return Task::batch(vec![
                        self.finish_mutation(&result),
                        self.start_ipsets_load(),
                        self.start_ipset_details_load(ipset_name),
                    ]);
                }
                Err(error) => {
                    self.ipset_view.entry_error = Some(error.to_string());
                    return self.finish_mutation(&Err(error));
                }
            },
            Message::IpSetDeleted { ipset_name, result } => match result {
                Ok(()) => {
                    self.runtime_reload_needed = true;
                    if self.ipset_view.selected.as_deref() == Some(ipset_name.as_str()) {
                        self.ipset_view.selected = None;
                        self.ipset_view.details = None;
                        self.ipset_view.entry_input.clear();
                        self.ipset_view.entry_error = None;
                    }
                    return Task::batch(vec![
                        self.finish_mutation(&result),
                        self.start_ipsets_load(),
                    ]);
                }
                Err(error) => {
                    self.ipset_view.entry_error = Some(error.to_string());
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
    fn mutation_pending(&self) -> bool {
        self.pending_operation.is_some()
    }

    fn begin_mutation(&mut self, operation: String) -> bool {
        if self.mutation_pending() {
            return false;
        }
        self.dialogs.operation_error = None;
        self.pending_operation = Some(operation);
        true
    }

    fn finish_mutation(
        &mut self,
        result: &Result<(), BrokerError>,
    ) -> Task<cosmic::Action<Message>> {
        let operation = self
            .pending_operation
            .take()
            .unwrap_or_else(|| fl!("operation-change"));
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
        let toast = self.toasts.push(toast).map(cosmic::Action::App);
        if self.configuration_coordinator.has_pending()
            && !self.configuration_coordinator.is_refreshing()
            && self.configuration_coordinator.finish_refresh()
        {
            return Task::batch(vec![
                toast,
                self.start_configuration_refresh(ConfigurationEvent::Reloaded),
            ]);
        }
        toast
    }

    fn schedule_configuration_refresh(
        &mut self,
        event: ConfigurationEvent,
    ) -> Task<cosmic::Action<Message>> {
        let blocked = self.mutation_pending()
            || matches!(
                self.zone_reconciliation,
                ZoneReconciliationState::Loading { .. }
            );
        match self.configuration_coordinator.request_refresh(blocked) {
            RefreshRequest::Start => self.start_configuration_refresh(event),
            RefreshRequest::Coalesced => Task::none(),
        }
    }

    fn start_configuration_refresh(
        &mut self,
        event: ConfigurationEvent,
    ) -> Task<cosmic::Action<Message>> {
        match event {
            ConfigurationEvent::Reloaded => Task::batch(vec![
                self.start_firewalld_status_load(),
                self.start_zones_load(),
                self.start_ipsets_load(),
                self.start_services_load(),
                self.start_icmp_types_load(),
                self.start_interfaces_load(),
            ]),
            ConfigurationEvent::RuntimeZoneChanged { zone } => {
                let is_current = self.current_zone_name().as_deref() == Some(zone.as_str());
                if !is_current {
                    return self.finish_configuration_refresh();
                }
                self.start_zone_reconciliation(zone)
            }
            ConfigurationEvent::PermanentZoneUpdated { zone } => {
                let is_current = self.current_zone_name().as_deref() == Some(zone.as_str());
                if !is_current {
                    return self.finish_configuration_refresh();
                }
                self.start_zone_load(zone)
            }
            ConfigurationEvent::PermanentZoneRemoved { .. } => self.start_zones_load(),
            ConfigurationEvent::PermanentZoneRenamed { old_zone, new_zone } => {
                self.sidebar.preserve_zone_rename(&old_zone, &new_zone);
                self.start_zones_load()
            }
        }
    }

    fn finish_configuration_refresh(&mut self) -> Task<cosmic::Action<Message>> {
        if self.configuration_coordinator.finish_refresh() {
            self.start_configuration_refresh(ConfigurationEvent::Reloaded)
        } else {
            Task::none()
        }
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
        match &self.zone_view {
            ZoneViewState::Ready(details) => Some(details.name.clone()),
            _ => None,
        }
    }

    fn validate_interface_value(&mut self) -> bool {
        let interface = self.dialogs.interface.interface.trim();
        match validate_interface_name(interface) {
            Ok(()) => {
                self.dialogs.interface.error = None;
                true
            }
            Err(message) => {
                self.dialogs.interface.error = Some(localized_validation_error(message));
                false
            }
        }
    }

    fn handle_nav_select(&mut self, id: nav_bar::Id) -> Task<cosmic::Action<Message>> {
        self.sidebar.activate(id);

        let task = match self.sidebar.active_item() {
            Some(SidebarItem::Zone { name, .. }) => self.start_zone_load(name.clone()),
            Some(SidebarItem::IpSets) => self.start_ipsets_load(),
            _ => {
                self.zone_view = ZoneViewState::Empty;
                Task::none()
            }
        };

        Task::batch(vec![self.update_title(), task])
    }

    fn handle_nav_menu_action(&mut self, action: NavMenuAction) -> Task<cosmic::Action<Message>> {
        match action {
            NavMenuAction::Open(id) => self.handle_nav_select(id),
            NavMenuAction::Activate(id) => {
                if self.sidebar.zone_name_for_id(id).is_none() {
                    return Task::none();
                }
                let task = self.handle_nav_select(id);
                self.context_page = ContextPage::AddInterface;
                self.core.window.show_context = true;
                self.reset_dialog_for_context(ContextPage::AddInterface);
                Task::batch(vec![task, self.start_interfaces_load()])
            }
            NavMenuAction::SetDefault(id) => {
                let Some(zone_name) = self.sidebar.zone_name_for_id(id) else {
                    return Task::none();
                };
                self.start_default_zone_set(zone_name)
            }
            NavMenuAction::Delete(id) => {
                let Some(zone_name) = self.sidebar.zone_name_for_id(id) else {
                    return Task::none();
                };
                self.confirmation = Some(Confirmation::DeleteZone(zone_name));
                Task::none()
            }
        }
    }

    fn handle_zone_action(&mut self, action: ZoneViewAction) -> Task<cosmic::Action<Message>> {
        if self.mutation_pending() {
            return Task::none();
        }
        match &action {
            ZoneViewAction::AddService => {
                self.open_context_page(ContextPage::AddService);
                return self.start_services_load();
            }
            ZoneViewAction::SetMasquerade(enabled) => {
                let Some(zone_name) = self.current_zone_name() else {
                    return Task::none();
                };
                return self.start_masquerade_set(zone_name, *enabled);
            }
            ZoneViewAction::SetIcmpBlockInversion(enabled) => {
                let Some(zone_name) = self.current_zone_name() else {
                    return Task::none();
                };
                return self.start_icmp_inversion_set(zone_name, *enabled);
            }
            ZoneViewAction::StartFirewalld => {
                return self.start_firewalld_control(true);
            }
            ZoneViewAction::StopFirewalld => {
                self.confirmation = Some(Confirmation::StopFirewalld);
                return Task::none();
            }
            ZoneViewAction::ApplyPermanentConfiguration => {
                self.confirmation = Some(Confirmation::ApplyPermanentConfiguration);
                return Task::none();
            }
            ZoneViewAction::SaveRuntimeConfiguration => {
                self.confirmation = Some(Confirmation::SaveRuntimeConfiguration);
                return Task::none();
            }
            ZoneViewAction::ReviewReconciliation => {
                self.open_context_page(ContextPage::ReviewReconciliation);
                return Task::none();
            }
            ZoneViewAction::RefreshReconciliation => {
                let Some(zone_name) = self.current_zone_name() else {
                    return Task::none();
                };
                if self.firewalld_status != FirewalldStatus::Active {
                    self.zone_reconciliation = ZoneReconciliationState::Unavailable {
                        zone: Some(zone_name),
                    };
                    return Task::none();
                }
                return self.start_zone_reconciliation(zone_name);
            }
            ZoneViewAction::AddInterface => {
                self.open_context_page(ContextPage::AddInterface);
                return self.start_interfaces_load();
            }
            ZoneViewAction::AddPort { forwarding } => {
                self.open_context_page(ContextPage::AddPort);
                self.dialogs.port.forwarding = *forwarding;
                return Task::none();
            }
            ZoneViewAction::AddSource => {
                self.open_context_page(ContextPage::AddSource);
                return Task::none();
            }
            ZoneViewAction::AddIcmpBlock => {
                self.open_context_page(ContextPage::AddIcmp);
                return self.start_icmp_types_load();
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

        let zone_name = match &self.zone_view {
            ZoneViewState::Ready(details) => details.name.clone(),
            _ => return Task::none(),
        };

        self.start_zone_item_remove(zone_name, action)
    }

    fn handle_dialog_message(&mut self, message: DialogMessage) -> Task<cosmic::Action<Message>> {
        if self.mutation_pending() && matches!(message, DialogMessage::Submit(_)) {
            return Task::none();
        }
        match message {
            DialogMessage::ZoneNameChanged(value) => {
                self.dialogs.zone.name = value;
            }
            DialogMessage::ZoneDescriptionChanged(value) => {
                self.dialogs.zone.description = value;
            }
            DialogMessage::ZoneTargetSelected(index) => {
                self.dialogs.zone.target = target_from_index(index);
            }
            DialogMessage::ServiceSearchChanged(value) => {
                self.dialogs.service.search = value;
            }
            DialogMessage::ServiceSelected(service) => {
                let Some(zone_name) = self.current_zone_name() else {
                    self.dialogs.operation_error = Some(fl!("error-select-zone-first"));
                    return Task::none();
                };
                let already_enabled = matches!(
                    &self.zone_view,
                    ZoneViewState::Ready(details) if details.services.contains(&service)
                );
                if already_enabled {
                    self.dialogs.operation_error = Some(fl!("error-service-already-enabled"));
                    return Task::none();
                }
                return self.start_service_add(zone_name, service);
            }
            DialogMessage::PortNumberChanged(value) => {
                self.dialogs.port.port = value;
                self.dialogs.port.port_touched = true;
            }
            DialogMessage::PortProtocolSelected(index) => {
                self.dialogs.port.protocol = crate::ui::dialog_drawers::protocol_from_index(index);
            }
            DialogMessage::PortForwardingToggled(value) => {
                self.dialogs.port.forwarding = value;
            }
            DialogMessage::PortForwardDestIpChanged(value) => {
                self.dialogs.port.dest_ip = value;
                self.dialogs.port.dest_ip_touched = true;
            }
            DialogMessage::PortForwardDestPortChanged(value) => {
                self.dialogs.port.dest_port = value;
                self.dialogs.port.dest_port_touched = true;
            }
            DialogMessage::InterfaceSelected(index) => {
                if index == 0 {
                    self.dialogs.interface.interface.clear();
                    self.dialogs.interface.error = None;
                } else if let Some(interface) = self.interface_options.get(index - 1) {
                    self.dialogs.interface.interface = interface.clone();
                    self.validate_interface_value();
                }
            }
            DialogMessage::InterfaceNameChanged(value) => {
                self.dialogs.interface.interface = value;
                self.validate_interface_value();
            }
            DialogMessage::SourceAddressChanged(value) => {
                self.dialogs.source.source = value;
                self.dialogs.source.touched = true;
            }
            DialogMessage::IcmpSearchChanged(value) => {
                self.dialogs.icmp.search = value;
            }
            DialogMessage::IcmpSelected(icmp_type) => {
                let Some(zone_name) = self.current_zone_name() else {
                    self.dialogs.operation_error = Some(fl!("error-select-zone-first"));
                    return Task::none();
                };
                let already_blocked = matches!(
                    &self.zone_view,
                    ZoneViewState::Ready(details) if details.icmp_blocks.contains(&icmp_type)
                );
                if already_blocked {
                    self.dialogs.operation_error = Some(fl!("error-icmp-already-blocked"));
                    return Task::none();
                }
                return self.start_icmp_add(zone_name, icmp_type);
            }
            DialogMessage::RichRuleRawModeToggled(value) => {
                self.dialogs.rich_rule.raw_mode = value;
            }
            DialogMessage::RichRuleRawChanged(value) => {
                self.dialogs.rich_rule.raw_rule = value;
            }
            DialogMessage::RichRuleFamilySelected(value) => {
                self.dialogs.rich_rule.family = value;
            }
            DialogMessage::RichRuleSourceChanged(value) => {
                self.dialogs.rich_rule.source = value;
            }
            DialogMessage::RichRuleSourceInvertToggled(value) => {
                self.dialogs.rich_rule.source_invert = value;
            }
            DialogMessage::RichRuleDestinationChanged(value) => {
                self.dialogs.rich_rule.destination = value;
            }
            DialogMessage::RichRuleDestinationInvertToggled(value) => {
                self.dialogs.rich_rule.destination_invert = value;
            }
            DialogMessage::RichRuleElementSelected(value) => {
                self.dialogs.rich_rule.element = value;
                self.dialogs.rich_rule.element_value.clear();
            }
            DialogMessage::RichRuleElementValueChanged(value) => {
                self.dialogs.rich_rule.element_value = value;
            }
            DialogMessage::RichRulePortProtocolSelected(value) => {
                self.dialogs.rich_rule.port_protocol =
                    crate::ui::dialog_drawers::protocol_from_index(value);
            }
            DialogMessage::RichRuleActionSelected(value) => {
                self.dialogs.rich_rule.action = value;
            }
            DialogMessage::RichRuleRejectTypeChanged(value) => {
                self.dialogs.rich_rule.reject_type = value;
            }
            DialogMessage::RichRuleMarkChanged(value) => {
                self.dialogs.rich_rule.mark = value;
            }
            DialogMessage::IpSetNameChanged(value) => {
                self.dialogs.ipset.name = value;
                self.dialogs.ipset.name_touched = true;
            }
            DialogMessage::IpSetTypeSelected(index) => {
                self.dialogs.ipset.ipset_type = crate::ui::dialog_drawers::ipset_from_index(index);
            }
            DialogMessage::IpSetEntriesChanged(value) => {
                self.dialogs.ipset.entries = value;
                self.dialogs.ipset.entries_touched = true;
            }
            DialogMessage::Submit(DialogKind::Zone) => {
                let name = self.dialogs.zone.name.trim().to_string();
                let description = self.dialogs.zone.description.trim().to_string();
                let target = self.dialogs.zone.target.clone();
                if name.is_empty() {
                    self.dialogs.operation_error = Some(fl!("error-required-field"));
                    return Task::none();
                }
                return self.start_zone_create(name, description, target);
            }
            DialogMessage::Submit(DialogKind::Service) => {
                return Task::none();
            }
            DialogMessage::Submit(DialogKind::Port) => {
                self.dialogs.port.port_touched = true;
                self.dialogs.port.dest_ip_touched = true;
                self.dialogs.port.dest_port_touched = true;
                if !self.dialogs.port.is_valid() {
                    self.dialogs.operation_error = Some(fl!("validation-fix-fields"));
                    return Task::none();
                }
                let port = self.dialogs.port.port.trim().to_string();
                let protocol = self.dialogs.port.protocol.trim().to_string();
                let forwarding = self.dialogs.port.forwarding;
                let dest_ip = self.dialogs.port.dest_ip.trim().to_string();
                let dest_port = self.dialogs.port.dest_port.trim().to_string();
                let Some(zone_name) = self.current_zone_name() else {
                    self.dialogs.operation_error = Some(fl!("error-select-zone-first"));
                    return Task::none();
                };
                if forwarding {
                    return self
                        .start_forward_port_add(zone_name, port, protocol, dest_port, dest_ip);
                }
                return self.start_port_add(zone_name, port, protocol);
            }
            DialogMessage::Submit(DialogKind::Interface) => {
                if !self.validate_interface_value() {
                    return Task::none();
                }
                let interface = self.dialogs.interface.interface.trim().to_string();
                let Some(zone_name) = self.current_zone_name() else {
                    self.dialogs.operation_error = Some(fl!("error-select-zone-first"));
                    return Task::none();
                };
                return self.start_interface_add(zone_name, interface);
            }
            DialogMessage::Submit(DialogKind::Source) => {
                self.dialogs.source.touched = true;
                if !self.dialogs.source.is_valid() {
                    self.dialogs.operation_error = Some(fl!("validation-fix-fields"));
                    return Task::none();
                }
                let source = self.dialogs.source.source.trim().to_string();
                let Some(zone_name) = self.current_zone_name() else {
                    self.dialogs.operation_error = Some(fl!("error-select-zone-first"));
                    return Task::none();
                };
                return self.start_source_add(zone_name, source);
            }
            DialogMessage::Submit(DialogKind::Icmp) => {
                return Task::none();
            }
            DialogMessage::Submit(DialogKind::RichRule) => {
                let Ok(rule) = self.dialogs.rich_rule.generated_rule() else {
                    self.dialogs.operation_error = Some(fl!("validation-fix-fields"));
                    return Task::none();
                };
                let Some(zone_name) = self.current_zone_name() else {
                    self.dialogs.operation_error = Some(fl!("error-select-zone-first"));
                    return Task::none();
                };
                return self.start_rich_rule_add(zone_name, rule);
            }
            DialogMessage::Submit(DialogKind::IpSet) => {
                self.dialogs.ipset.name_touched = true;
                self.dialogs.ipset.entries_touched = true;
                if !self.dialogs.ipset.is_valid() {
                    self.dialogs.operation_error = Some(fl!("validation-fix-fields"));
                    return Task::none();
                }
                let name = self.dialogs.ipset.name.trim().to_string();
                let ipset_type = self.dialogs.ipset.ipset_type.trim().to_string();
                let entries = split_ipset_entries(&self.dialogs.ipset.entries);
                return self.start_ipset_create(name, ipset_type, entries);
            }
            DialogMessage::Cancel(kind) => {
                self.dialogs.reset(kind);
                self.close_context_drawer();
            }
        }
        Task::none()
    }

    fn handle_ipset_action(&mut self, action: IpSetViewAction) -> Task<cosmic::Action<Message>> {
        if self.mutation_pending()
            && matches!(
                action,
                IpSetViewAction::AddEntry | IpSetViewAction::RemoveEntry(_)
            )
        {
            return Task::none();
        }
        match action {
            IpSetViewAction::Select(name) => {
                self.ipset_view.selected = Some(name.clone());
                self.ipset_view.details = None;
                self.ipset_view.entry_error = None;
                self.ipset_view.entry_input.clear();
                return self.start_ipset_details_load(name);
            }
            IpSetViewAction::EntryInputChanged(value) => {
                self.ipset_view.entry_input = value;
                self.ipset_view.entry_error = self
                    .ipset_view
                    .details
                    .as_ref()
                    .and_then(|details| {
                        let input = self.ipset_view.entry_input.trim();
                        (!input.is_empty())
                            .then(|| validate_ipset_entry(input, &details.ipset_type).err())
                            .flatten()
                    })
                    .map(localized_validation_error);
            }
            IpSetViewAction::AddEntry => {
                let Some(ipset_name) = self.ipset_view.selected.clone() else {
                    return Task::none();
                };
                let entry = self.ipset_view.entry_input.trim();
                let Some(details) = &self.ipset_view.details else {
                    return Task::none();
                };
                if let Err(error) = validate_ipset_entry(entry, &details.ipset_type) {
                    self.ipset_view.entry_error = Some(localized_validation_error(error));
                    return Task::none();
                }
                return self.start_ipset_entry_add(ipset_name, entry.to_string());
            }
            IpSetViewAction::RemoveEntry(entry) => {
                let Some(ipset_name) = self.ipset_view.selected.clone() else {
                    return Task::none();
                };
                return self.start_ipset_entry_remove(ipset_name, entry);
            }
            IpSetViewAction::DeleteSelected => {
                let Some(ipset_name) = self.ipset_view.selected.clone() else {
                    return Task::none();
                };
                self.confirmation = Some(Confirmation::DeleteIpSet(ipset_name));
            }
        }

        Task::none()
    }

    async fn load_zones() -> Result<Vec<String>, BrokerError> {
        let broker = FwdBroker::get().await?;
        broker.get_zones().await
    }

    async fn load_zone_details(zone_name: String) -> Result<ZoneDetails, BrokerError> {
        let broker = FwdBroker::get().await?;
        broker.get_zone_details(&zone_name).await
    }

    async fn load_zone_reconciliation(
        zone_name: String,
    ) -> Result<ZoneReconciliationData, BrokerError> {
        let broker = FwdBroker::get().await?;
        broker.reconcile_zone(&zone_name).await
    }

    async fn load_default_zone() -> Result<String, BrokerError> {
        let broker = FwdBroker::get().await?;
        broker.get_default_zone().await
    }

    async fn load_active_zones() -> Result<HashSet<String>, BrokerError> {
        let broker = FwdBroker::get().await?;
        broker.get_active_zones().await
    }

    async fn load_interfaces() -> Result<Vec<String>, BrokerError> {
        let broker = FwdBroker::get().await?;
        broker.get_interfaces().await
    }

    async fn load_services() -> Result<Vec<String>, BrokerError> {
        let broker = FwdBroker::get().await?;
        broker.get_services().await
    }

    async fn load_icmp_types() -> Result<Vec<IcmpTypeInfo>, BrokerError> {
        let broker = FwdBroker::get().await?;
        broker.get_icmp_types().await
    }

    async fn add_service(zone_name: String, service: String) -> Result<(), BrokerError> {
        let broker = FwdBroker::get().await?;
        broker.add_service(&zone_name, &service).await
    }

    async fn load_firewalld_status() -> Result<FirewalldStatus, BrokerError> {
        let broker = FwdBroker::get().await?;
        broker.firewalld_status().await
    }

    async fn set_masquerade(zone_name: String, enabled: bool) -> Result<(), BrokerError> {
        let broker = FwdBroker::get().await?;
        broker.set_masquerade(&zone_name, enabled).await
    }

    async fn set_icmp_block_inversion(zone_name: String, enabled: bool) -> Result<(), BrokerError> {
        let broker = FwdBroker::get().await?;
        broker.set_icmp_block_inversion(&zone_name, enabled).await
    }

    async fn control_firewalld(start: bool) -> Result<(), BrokerError> {
        let broker = FwdBroker::get().await?;
        if start {
            broker.start_firewalld().await
        } else {
            broker.stop_firewalld().await
        }
    }

    async fn apply_permanent_configuration() -> Result<(), BrokerError> {
        let broker = FwdBroker::get().await?;
        broker.apply_permanent_configuration().await
    }

    async fn persist_runtime_configuration() -> Result<(), BrokerError> {
        let broker = FwdBroker::get().await?;
        broker.persist_runtime_configuration().await
    }

    async fn set_default_zone(zone_name: String) -> Result<(), BrokerError> {
        let broker = FwdBroker::get().await?;
        broker.set_default_zone(&zone_name).await
    }

    async fn remove_zone(zone_name: String) -> Result<(), BrokerError> {
        let broker = FwdBroker::get().await?;
        broker.remove_zone(&zone_name).await
    }

    async fn add_zone(
        name: String,
        description: String,
        target: ZoneTarget,
    ) -> Result<(), BrokerError> {
        let broker = FwdBroker::get().await?;
        broker.add_zone(&name, &description, &target).await
    }

    async fn add_port(
        zone_name: String,
        port: String,
        protocol: String,
    ) -> Result<(), BrokerError> {
        let broker = FwdBroker::get().await?;
        broker.add_port(&zone_name, &port, &protocol).await
    }

    async fn add_forward_port(
        zone_name: String,
        port: String,
        protocol: String,
        to_port: String,
        to_addr: String,
    ) -> Result<(), BrokerError> {
        let broker = FwdBroker::get().await?;
        broker
            .add_forward_port(&zone_name, &port, &protocol, &to_port, &to_addr)
            .await
    }

    async fn add_interface(zone_name: String, interface: String) -> Result<(), BrokerError> {
        let broker = FwdBroker::get().await?;
        broker.add_interface(&zone_name, &interface).await
    }

    async fn add_source(zone_name: String, source: String) -> Result<(), BrokerError> {
        let broker = FwdBroker::get().await?;
        broker.add_source(&zone_name, &source).await
    }

    async fn add_icmp_block(zone_name: String, icmp: String) -> Result<(), BrokerError> {
        let broker = FwdBroker::get().await?;
        broker.add_icmp_block(&zone_name, &icmp).await
    }

    async fn add_rich_rule(zone_name: String, rule: String) -> Result<(), BrokerError> {
        let broker = FwdBroker::get().await?;
        broker.add_rich_rule(&zone_name, &rule).await
    }

    async fn remove_service(zone_name: String, service: String) -> Result<(), BrokerError> {
        let broker = FwdBroker::get().await?;
        broker.remove_service(&zone_name, &service).await
    }

    async fn remove_interface(zone_name: String, interface: String) -> Result<(), BrokerError> {
        let broker = FwdBroker::get().await?;
        broker.remove_interface(&zone_name, &interface).await
    }

    async fn remove_source(zone_name: String, source: String) -> Result<(), BrokerError> {
        let broker = FwdBroker::get().await?;
        broker.remove_source(&zone_name, &source).await
    }

    async fn remove_port(
        zone_name: String,
        port: String,
        protocol: String,
    ) -> Result<(), BrokerError> {
        let broker = FwdBroker::get().await?;
        broker.remove_port(&zone_name, &port, &protocol).await
    }

    async fn remove_forward_port(
        zone_name: String,
        port: String,
        protocol: String,
        to_port: String,
        to_addr: String,
    ) -> Result<(), BrokerError> {
        let broker = FwdBroker::get().await?;
        broker
            .remove_forward_port(&zone_name, &port, &protocol, &to_port, &to_addr)
            .await
    }

    async fn remove_source_port(
        zone_name: String,
        port: String,
        protocol: String,
    ) -> Result<(), BrokerError> {
        let broker = FwdBroker::get().await?;
        broker
            .remove_source_port(&zone_name, &port, &protocol)
            .await
    }

    async fn remove_icmp_block(zone_name: String, icmp: String) -> Result<(), BrokerError> {
        let broker = FwdBroker::get().await?;
        broker.remove_icmp_block(&zone_name, &icmp).await
    }

    async fn remove_rich_rule(zone_name: String, rule: String) -> Result<(), BrokerError> {
        let broker = FwdBroker::get().await?;
        broker.remove_rich_rule(&zone_name, &rule).await
    }

    async fn remove_ipset_entry(ipset_name: String, entry: String) -> Result<(), BrokerError> {
        let broker = FwdBroker::get().await?;
        broker.remove_ipset_entry(&ipset_name, &entry).await
    }

    async fn remove_ipset(ipset_name: String) -> Result<(), BrokerError> {
        let broker = FwdBroker::get().await?;
        broker.remove_ipset(&ipset_name).await
    }

    async fn load_ipsets() -> Result<Vec<String>, BrokerError> {
        let broker = FwdBroker::get().await?;
        broker.get_ipsets().await
    }

    async fn load_ipset_details(ipset_name: String) -> Result<IpSetDetails, BrokerError> {
        let broker = FwdBroker::get().await?;
        broker.get_ipset_details(&ipset_name).await
    }

    async fn add_ipset_entry(ipset_name: String, entry: String) -> Result<(), BrokerError> {
        let broker = FwdBroker::get().await?;
        broker.add_ipset_entry(&ipset_name, &entry).await
    }

    async fn create_ipset(
        name: String,
        ipset_type: String,
        entries: Vec<String>,
    ) -> Result<(), BrokerError> {
        let broker = FwdBroker::get().await?;
        broker.create_ipset(&name, &ipset_type, entries).await
    }

    fn start_zones_load(&mut self) -> Task<cosmic::Action<Message>> {
        self.sidebar.set_loading();
        Task::perform(Self::load_zones(), |result| {
            cosmic::Action::from(Message::ZonesLoaded(result))
        })
    }

    fn start_default_zone_load(&mut self) -> Task<cosmic::Action<Message>> {
        Task::perform(Self::load_default_zone(), |result| {
            cosmic::Action::from(Message::DefaultZoneLoaded(result))
        })
    }

    fn start_active_zones_load(&mut self) -> Task<cosmic::Action<Message>> {
        Task::perform(Self::load_active_zones(), |result| {
            cosmic::Action::from(Message::ActiveZonesLoaded(result))
        })
    }

    fn start_interfaces_load(&mut self) -> Task<cosmic::Action<Message>> {
        self.interface_loading = true;
        self.interface_error = None;
        self.interface_options.clear();
        Task::perform(Self::load_interfaces(), |result| {
            cosmic::Action::from(Message::InterfacesLoaded(result))
        })
    }

    fn start_services_load(&mut self) -> Task<cosmic::Action<Message>> {
        self.service_loading = true;
        self.service_error = None;
        Task::perform(Self::load_services(), |result| {
            cosmic::Action::from(Message::ServicesLoaded(result))
        })
    }

    fn start_icmp_types_load(&mut self) -> Task<cosmic::Action<Message>> {
        self.icmp_loading = true;
        self.icmp_error = None;
        Task::perform(Self::load_icmp_types(), |result| {
            cosmic::Action::from(Message::IcmpTypesLoaded(result))
        })
    }

    fn start_service_add(
        &mut self,
        zone_name: String,
        service: String,
    ) -> Task<cosmic::Action<Message>> {
        if !self.begin_mutation(fl!("operation-add-service")) {
            return Task::none();
        }
        let zone_name_for_task = zone_name.clone();
        Task::perform(Self::add_service(zone_name, service), move |result| {
            cosmic::Action::from(Message::ZoneItemAdded {
                zone_name: zone_name_for_task.clone(),
                result,
            })
        })
    }

    fn start_firewalld_status_load(&mut self) -> Task<cosmic::Action<Message>> {
        self.firewalld_status = FirewalldStatus::Loading;
        Task::perform(Self::load_firewalld_status(), |result| {
            cosmic::Action::from(Message::FirewalldStatusLoaded(result))
        })
    }

    fn start_masquerade_set(
        &mut self,
        zone_name: String,
        enabled: bool,
    ) -> Task<cosmic::Action<Message>> {
        if !self.begin_mutation(fl!("operation-set-masquerading")) {
            return Task::none();
        }
        let zone_name_for_task = zone_name.clone();
        Task::perform(Self::set_masquerade(zone_name, enabled), move |result| {
            cosmic::Action::from(Message::ZoneItemAdded {
                zone_name: zone_name_for_task.clone(),
                result,
            })
        })
    }

    fn start_icmp_inversion_set(
        &mut self,
        zone_name: String,
        enabled: bool,
    ) -> Task<cosmic::Action<Message>> {
        if !self.begin_mutation(fl!("operation-set-icmp-inversion")) {
            return Task::none();
        }
        let zone_name_for_task = zone_name.clone();
        Task::perform(
            Self::set_icmp_block_inversion(zone_name, enabled),
            move |result| {
                cosmic::Action::from(Message::ZoneItemAdded {
                    zone_name: zone_name_for_task.clone(),
                    result,
                })
            },
        )
    }

    fn start_firewalld_control(&mut self, start: bool) -> Task<cosmic::Action<Message>> {
        let operation = if start {
            fl!("operation-start-firewalld")
        } else {
            fl!("operation-stop-firewalld")
        };
        if !self.begin_mutation(operation) {
            return Task::none();
        }
        self.firewalld_status = FirewalldStatus::Loading;
        Task::perform(Self::control_firewalld(start), |result| {
            cosmic::Action::from(Message::FirewalldControlFinished {
                apply_permanent: false,
                result,
            })
        })
    }

    fn start_permanent_apply(&mut self) -> Task<cosmic::Action<Message>> {
        if !self.begin_mutation(fl!("operation-apply-permanent")) {
            return Task::none();
        }
        Task::perform(Self::apply_permanent_configuration(), |result| {
            cosmic::Action::from(Message::FirewalldControlFinished {
                apply_permanent: true,
                result,
            })
        })
    }

    fn start_runtime_configuration_persist(&mut self) -> Task<cosmic::Action<Message>> {
        if !self.begin_mutation(fl!("operation-save-runtime")) {
            return Task::none();
        }
        Task::perform(Self::persist_runtime_configuration(), |result| {
            cosmic::Action::from(Message::RuntimeConfigurationPersisted(result))
        })
    }

    fn start_default_zone_set(&mut self, zone_name: String) -> Task<cosmic::Action<Message>> {
        if !self.begin_mutation(fl!("operation-set-default-zone")) {
            return Task::none();
        }
        Task::perform(Self::set_default_zone(zone_name), |result| {
            cosmic::Action::from(Message::DefaultZoneSet(result))
        })
    }

    fn start_zone_create(
        &mut self,
        name: String,
        description: String,
        target: ZoneTarget,
    ) -> Task<cosmic::Action<Message>> {
        if !self.begin_mutation(fl!("operation-create-zone")) {
            return Task::none();
        }
        let zone_name_for_task = name.clone();
        Task::perform(Self::add_zone(name, description, target), move |result| {
            cosmic::Action::from(Message::ZoneCreated {
                zone_name: zone_name_for_task.clone(),
                result,
            })
        })
    }

    fn start_port_add(
        &mut self,
        zone_name: String,
        port: String,
        protocol: String,
    ) -> Task<cosmic::Action<Message>> {
        if !self.begin_mutation(fl!("operation-add-port")) {
            return Task::none();
        }
        let zone_name_for_task = zone_name.clone();
        Task::perform(Self::add_port(zone_name, port, protocol), move |result| {
            cosmic::Action::from(Message::ZoneItemAdded {
                zone_name: zone_name_for_task.clone(),
                result,
            })
        })
    }

    fn start_forward_port_add(
        &mut self,
        zone_name: String,
        port: String,
        protocol: String,
        to_port: String,
        to_addr: String,
    ) -> Task<cosmic::Action<Message>> {
        if !self.begin_mutation(fl!("operation-add-forward-port")) {
            return Task::none();
        }
        let zone_name_for_task = zone_name.clone();
        Task::perform(
            Self::add_forward_port(zone_name, port, protocol, to_port, to_addr),
            move |result| {
                cosmic::Action::from(Message::ZoneItemAdded {
                    zone_name: zone_name_for_task.clone(),
                    result,
                })
            },
        )
    }

    fn start_interface_add(
        &mut self,
        zone_name: String,
        interface: String,
    ) -> Task<cosmic::Action<Message>> {
        if !self.begin_mutation(fl!("operation-add-interface")) {
            return Task::none();
        }
        let zone_name_for_task = zone_name.clone();
        Task::perform(Self::add_interface(zone_name, interface), move |result| {
            cosmic::Action::from(Message::ZoneItemAdded {
                zone_name: zone_name_for_task.clone(),
                result,
            })
        })
    }

    fn start_source_add(
        &mut self,
        zone_name: String,
        source: String,
    ) -> Task<cosmic::Action<Message>> {
        if !self.begin_mutation(fl!("operation-add-source")) {
            return Task::none();
        }
        let zone_name_for_task = zone_name.clone();
        Task::perform(Self::add_source(zone_name, source), move |result| {
            cosmic::Action::from(Message::ZoneItemAdded {
                zone_name: zone_name_for_task.clone(),
                result,
            })
        })
    }

    fn start_icmp_add(&mut self, zone_name: String, icmp: String) -> Task<cosmic::Action<Message>> {
        if !self.begin_mutation(fl!("operation-add-icmp")) {
            return Task::none();
        }
        let zone_name_for_task = zone_name.clone();
        Task::perform(Self::add_icmp_block(zone_name, icmp), move |result| {
            cosmic::Action::from(Message::ZoneItemAdded {
                zone_name: zone_name_for_task.clone(),
                result,
            })
        })
    }

    fn start_rich_rule_add(
        &mut self,
        zone_name: String,
        rule: String,
    ) -> Task<cosmic::Action<Message>> {
        if !self.begin_mutation(fl!("operation-add-rich-rule")) {
            return Task::none();
        }
        let zone_name_for_task = zone_name.clone();
        Task::perform(Self::add_rich_rule(zone_name, rule), move |result| {
            cosmic::Action::from(Message::ZoneItemAdded {
                zone_name: zone_name_for_task.clone(),
                result,
            })
        })
    }

    fn start_zone_delete(&mut self, zone_name: String) -> Task<cosmic::Action<Message>> {
        if !self.begin_mutation(fl!("operation-delete-zone")) {
            return Task::none();
        }
        let zone_name_for_task = zone_name.clone();
        Task::perform(Self::remove_zone(zone_name), move |result| {
            cosmic::Action::from(Message::ZoneDeleted {
                zone_name: zone_name_for_task.clone(),
                result,
            })
        })
    }

    fn start_zone_item_remove(
        &mut self,
        zone_name: String,
        action: ZoneViewAction,
    ) -> Task<cosmic::Action<Message>> {
        if !self.begin_mutation(fl!("operation-remove-zone-item")) {
            return Task::none();
        }
        let zone_name_for_task = zone_name.clone();
        match action {
            ZoneViewAction::SetMasquerade(_)
            | ZoneViewAction::SetIcmpBlockInversion(_)
            | ZoneViewAction::StartFirewalld
            | ZoneViewAction::StopFirewalld
            | ZoneViewAction::ApplyPermanentConfiguration
            | ZoneViewAction::SaveRuntimeConfiguration
            | ZoneViewAction::ReviewReconciliation
            | ZoneViewAction::RefreshReconciliation => Task::none(),
            ZoneViewAction::AddService
            | ZoneViewAction::AddInterface
            | ZoneViewAction::AddPort { .. }
            | ZoneViewAction::AddSource
            | ZoneViewAction::AddIcmpBlock
            | ZoneViewAction::AddRichRule => Task::none(),
            ZoneViewAction::RemoveService(service) => {
                Task::perform(Self::remove_service(zone_name, service), move |result| {
                    cosmic::Action::from(Message::ZoneItemRemoved {
                        zone_name: zone_name_for_task.clone(),
                        result,
                    })
                })
            }
            ZoneViewAction::RemoveInterface(interface) => Task::perform(
                Self::remove_interface(zone_name, interface),
                move |result| {
                    cosmic::Action::from(Message::ZoneItemRemoved {
                        zone_name: zone_name_for_task.clone(),
                        result,
                    })
                },
            ),
            ZoneViewAction::RemoveSource(source) => {
                Task::perform(Self::remove_source(zone_name, source), move |result| {
                    cosmic::Action::from(Message::ZoneItemRemoved {
                        zone_name: zone_name_for_task.clone(),
                        result,
                    })
                })
            }
            ZoneViewAction::RemovePort { port, protocol } => Task::perform(
                Self::remove_port(zone_name, port, protocol),
                move |result| {
                    cosmic::Action::from(Message::ZoneItemRemoved {
                        zone_name: zone_name_for_task.clone(),
                        result,
                    })
                },
            ),
            ZoneViewAction::RemoveForwardPort {
                port,
                protocol,
                to_port,
                to_addr,
            } => Task::perform(
                Self::remove_forward_port(zone_name, port, protocol, to_port, to_addr),
                move |result| {
                    cosmic::Action::from(Message::ZoneItemRemoved {
                        zone_name: zone_name_for_task.clone(),
                        result,
                    })
                },
            ),
            ZoneViewAction::RemoveSourcePort { port, protocol } => Task::perform(
                Self::remove_source_port(zone_name, port, protocol),
                move |result| {
                    cosmic::Action::from(Message::ZoneItemRemoved {
                        zone_name: zone_name_for_task.clone(),
                        result,
                    })
                },
            ),
            ZoneViewAction::RemoveIcmpBlock(icmp) => {
                Task::perform(Self::remove_icmp_block(zone_name, icmp), move |result| {
                    cosmic::Action::from(Message::ZoneItemRemoved {
                        zone_name: zone_name_for_task.clone(),
                        result,
                    })
                })
            }
            ZoneViewAction::RemoveRichRule(rule) => {
                Task::perform(Self::remove_rich_rule(zone_name, rule), move |result| {
                    cosmic::Action::from(Message::ZoneItemRemoved {
                        zone_name: zone_name_for_task.clone(),
                        result,
                    })
                })
            }
        }
    }

    fn start_zone_load(&mut self, zone_name: String) -> Task<cosmic::Action<Message>> {
        self.configuration_coordinator.selection_changed();
        self.zone_view = ZoneViewState::Loading {
            zone: zone_name.clone(),
        };
        self.zone_reconciliation = ZoneReconciliationState::Unavailable {
            zone: Some(zone_name.clone()),
        };

        let zone_name_for_task = zone_name.clone();
        Task::perform(Self::load_zone_details(zone_name), move |result| {
            cosmic::Action::from(Message::ZoneDetailsLoaded {
                zone_name: zone_name_for_task.clone(),
                result,
            })
        })
    }

    fn start_zone_reconciliation(&mut self, zone_name: String) -> Task<cosmic::Action<Message>> {
        let generation = self.configuration_coordinator.generation();
        self.zone_reconciliation = ZoneReconciliationState::Loading {
            zone: zone_name.clone(),
        };
        let zone_name_for_task = zone_name.clone();
        Task::perform(Self::load_zone_reconciliation(zone_name), move |result| {
            cosmic::Action::from(Message::ZoneReconciliationLoaded {
                zone_name: zone_name_for_task.clone(),
                generation,
                result: Box::new(result),
            })
        })
    }

    fn start_ipsets_load(&mut self) -> Task<cosmic::Action<Message>> {
        self.ipset_view.list_loading = true;
        Task::perform(Self::load_ipsets(), |result| {
            cosmic::Action::from(Message::IpSetsLoaded(result))
        })
    }

    fn start_ipset_details_load(&mut self, ipset_name: String) -> Task<cosmic::Action<Message>> {
        self.ipset_view.details_loading = true;
        let ipset_name_for_task = ipset_name.clone();
        Task::perform(Self::load_ipset_details(ipset_name), move |result| {
            cosmic::Action::from(Message::IpSetDetailsLoaded {
                ipset_name: ipset_name_for_task.clone(),
                result,
            })
        })
    }

    fn start_ipset_entry_add(
        &mut self,
        ipset_name: String,
        entry: String,
    ) -> Task<cosmic::Action<Message>> {
        if !self.begin_mutation(fl!("operation-add-ipset-entry")) {
            return Task::none();
        }
        let ipset_name_for_task = ipset_name.clone();
        Task::perform(Self::add_ipset_entry(ipset_name, entry), move |result| {
            cosmic::Action::from(Message::IpSetEntryAdded {
                ipset_name: ipset_name_for_task.clone(),
                result,
            })
        })
    }

    fn start_ipset_entry_remove(
        &mut self,
        ipset_name: String,
        entry: String,
    ) -> Task<cosmic::Action<Message>> {
        if !self.begin_mutation(fl!("operation-remove-ipset-entry")) {
            return Task::none();
        }
        let ipset_name_for_task = ipset_name.clone();
        Task::perform(Self::remove_ipset_entry(ipset_name, entry), move |result| {
            cosmic::Action::from(Message::IpSetEntryRemoved {
                ipset_name: ipset_name_for_task.clone(),
                result,
            })
        })
    }

    fn start_ipset_delete(&mut self, ipset_name: String) -> Task<cosmic::Action<Message>> {
        if !self.begin_mutation(fl!("operation-delete-ipset")) {
            return Task::none();
        }
        let ipset_name_for_task = ipset_name.clone();
        Task::perform(Self::remove_ipset(ipset_name), move |result| {
            cosmic::Action::from(Message::IpSetDeleted {
                ipset_name: ipset_name_for_task.clone(),
                result,
            })
        })
    }

    fn start_ipset_create(
        &mut self,
        ipset_name: String,
        ipset_type: String,
        entries: Vec<String>,
    ) -> Task<cosmic::Action<Message>> {
        if !self.begin_mutation(fl!("operation-create-ipset")) {
            return Task::none();
        }
        let ipset_name_for_task = ipset_name.clone();
        Task::perform(
            Self::create_ipset(ipset_name, ipset_type, entries),
            move |result| {
                cosmic::Action::from(Message::IpSetCreated {
                    ipset_name: ipset_name_for_task.clone(),
                    result,
                })
            },
        )
    }

    /// Updates the header and window titles.
    pub fn update_title(&mut self) -> Task<cosmic::Action<Message>> {
        let mut window_title = fl!("app-title");

        if let Some(page) = self.sidebar.active_label() {
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

fn split_ipset_entries(entries: &str) -> Vec<String> {
    entries
        .lines()
        .map(str::trim)
        .filter(|entry| !entry.is_empty())
        .map(|entry| entry.to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::split_ipset_entries;

    #[test]
    fn ipset_entry_lines_preserve_composite_tuple_commas() {
        assert_eq!(
            split_ipset_entries("192.0.2.1,443,198.51.100.2\n\n  2001:db8::1,53  \n"),
            vec!["192.0.2.1,443,198.51.100.2", "2001:db8::1,53",]
        );
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

fn configuration_event_messages(selected_zone: Option<String>) -> BoxStream<'static, Message> {
    Box::pin(async_stream::stream! {
        let broker = match FwdBroker::get().await {
            Ok(broker) => broker,
            Err(error) => {
                yield Message::ConfigurationEvent(Err(error));
                return;
            }
        };
        let mut events = broker.configuration_events(selected_zone);
        while let Some(event) = events.next().await {
            let failed = event.is_err();
            yield Message::ConfigurationEvent(event);
            if failed {
                return;
            }
        }
    })
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
        cosmic::Action::App(Message::NavMenuAction(*self))
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
            MenuAction::About => Message::ToggleContextPage(ContextPage::About),
            MenuAction::AddZone => Message::ToggleContextPage(ContextPage::AddZone),
            MenuAction::AddPort => Message::ToggleContextPage(ContextPage::AddPort),
            MenuAction::AddInterface => Message::ToggleContextPage(ContextPage::AddInterface),
            MenuAction::AddSource => Message::ToggleContextPage(ContextPage::AddSource),
            MenuAction::AddIcmp => Message::ToggleContextPage(ContextPage::AddIcmp),
            MenuAction::AddRichRule => Message::ToggleContextPage(ContextPage::AddRichRule),
            MenuAction::AddIpSet => Message::ToggleContextPage(ContextPage::AddIpSet),
        }
    }
}
