// SPDX-License-Identifier: MIT

use crate::config::Config;
use crate::core::{BrokerError, ConfigurationEvent, FirewalldStatus, FwdBroker};
use crate::fl;
use crate::models::{ZoneDetails, ZoneTarget};
use cosmic::app::context_drawer;
use cosmic::cosmic_config::{self, CosmicConfigEntry};
use cosmic::iced::alignment::{Horizontal, Vertical};
use cosmic::iced::{Length, Subscription};
use cosmic::prelude::*;
use cosmic::widget::{self, Toast, ToastId, about::About, menu, nav_bar};
use dialogs::{
    DialogKind, DialogMessage, DialogState, PortKind, drawer_cancel_footer,
    drawer_footer_with_submit, drawer_with_error, icmp_drawer, interface_drawer, ipset_drawer,
    localized_validation_error, port_drawer, rich_rule_drawer, service_drawer, source_drawer,
};
use ipsets::view_ipset_content;
use navigation::SidebarItem;
use reconciliation::reconciliation_drawer;
use std::collections::{HashMap, HashSet};
use zones::{ZoneViewAction, ZoneViewState, view_zone_content};

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
        outcome.append(outcome::Outcome::effect(app.start_zones_load()));
        outcome.append(outcome::Outcome::effect(app.start_firewalld_status_load()));
        let command = app.route(outcome);

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
        Some(self.navigation.nav_model())
    }

    /// The context menu to display for the active nav-bar item.
    fn nav_context_menu(&self) -> Option<Vec<menu::Tree<cosmic::Action<Self::Message>>>> {
        let context_id = self.navigation.nav_model().active();

        let Some(item) = self.navigation.item_for_id(context_id) else {
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
            .or(self.catalogs.interfaces.error());
        let can_submit_interface = self.dialogs.interface.error.is_none()
            && !self.dialogs.interface.interface.trim().is_empty();
        let can_submit = !self.mutation_pending();
        let error = self.dialogs.operation_error.as_deref();
        let enabled_services = match &self.zones {
            ZoneViewState::Ready(details) => details.services.as_slice(),
            _ => &[],
        };
        let blocked_icmp = match &self.zones {
            ZoneViewState::Ready(details) => details.icmp_blocks.as_slice(),
            _ => &[],
        };

        Some(match self.context_page {
            ContextPage::About => context_drawer::about(
                &self.about,
                |url| Message::Navigation(navigation::Message::LaunchUrl(url.to_string())),
                Message::Navigation(navigation::Message::ToggleContextPage(ContextPage::About)),
            ),
            ContextPage::ReviewReconciliation => context_drawer::context_drawer(
                reconciliation_drawer(
                    self.reconciliation.state(),
                    self.mutation_pending(),
                    self.dialogs.operation_error.as_deref(),
                    self.reconciliation.watch_warning(),
                    |action| Message::Zone(zones::Message::View(action)),
                ),
                Message::Navigation(navigation::Message::ToggleContextPage(
                    ContextPage::ReviewReconciliation,
                )),
            )
            .title(fl!("reconciliation-review-title")),
            ContextPage::AddZone => context_drawer::context_drawer(
                drawer_with_error(dialogs::zone_drawer(&self.dialogs.zone), error),
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
                        self.catalogs.services.items(),
                        enabled_services,
                        self.catalogs.services.is_loading(),
                        self.catalogs.services.error(),
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
            .title(port_drawer_title(self.dialogs.port.kind))
            .footer(drawer_footer_with_submit(
                DialogKind::Port,
                can_submit && self.dialogs.port.is_valid(),
            ))
            .map(Message::Dialog),
            ContextPage::AddInterface => context_drawer::context_drawer(
                drawer_with_error(
                    interface_drawer(
                        &self.dialogs.interface,
                        self.catalogs.interfaces.items(),
                        self.catalogs.interfaces.is_loading(),
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
                        self.catalogs.icmp_types.items(),
                        blocked_icmp,
                        self.catalogs.icmp_types.is_loading(),
                        self.catalogs.icmp_types.error(),
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
        let confirmation = self.operations.confirmation.as_ref()?;
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
        self.operations.pending.as_ref().map(|operation| {
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
        let content: Element<_> = match self.navigation.active_item() {
            Some(SidebarItem::IpSets) => {
                view_ipset_content(&self.ipsets, self.mutation_pending(), |action| {
                    Message::IpSet(ipsets::Message::View(action))
                })
            }
            _ => view_zone_content(
                &self.zones,
                &self.firewalld_status,
                self.reconciliation.state(),
                self.reconciliation.watch_warning(),
                self.mutation_pending(),
                |action| Message::Zone(zones::Message::View(action)),
            ),
        };

        widget::toaster(
            &self.operations.toasts,
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
                    Confirmation::DeleteZone(zone_name) => self.start_zone_delete(zone_name),
                    Confirmation::DeleteIpSet(ipset_name) => self.start_ipset_delete(ipset_name),
                    Confirmation::StopFirewalld => self.start_firewalld_control(false),
                    Confirmation::ApplyPermanentConfiguration => self.start_permanent_apply(),
                    Confirmation::SaveRuntimeConfiguration => {
                        self.start_runtime_configuration_persist()
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
                        return self.start_zone_reconciliation(details.name.clone());
                    }
                } else {
                    self.reconciliation
                        .set_unavailable(self.current_zone_name());
                }
            }
            Message::Zone(zones::Message::DaemonControlFinished(result)) => {
                return Task::batch(vec![
                    self.finish_mutation(&result),
                    self.start_firewalld_status_load(),
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
                        tasks.push(self.start_default_zone_load());
                        tasks.push(self.start_active_zones_load());
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
                    Some(SidebarItem::Zone { name, .. }) => self.start_zone_load(name.clone()),
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
                            return self.start_zone_reconciliation(zone_name);
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
                        self.start_default_zone_load(),
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
                        self.start_zones_load(),
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
                        self.start_zones_load(),
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
                    tasks.push(self.start_firewalld_status_load());
                }
                reconciliation::Request::RefreshZones => {
                    tasks.push(self.start_zones_load());
                }
                reconciliation::Request::RefreshIpSets => {
                    tasks.push(self.start_ipsets_load());
                }
                reconciliation::Request::RefreshCatalogs => {
                    tasks.extend([
                        self.start_services_load(),
                        self.start_icmp_types_load(),
                        self.start_interfaces_load(),
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
                self.navigation.preserve_zone_rename(&old_zone, &new_zone);
                self.start_zones_load()
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
            Some(SidebarItem::Zone { name, .. }) => self.start_zone_load(name.clone()),
            Some(SidebarItem::IpSets) => self.start_ipsets_load(),
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
                Task::batch(vec![task, self.start_interfaces_load()])
            }
            NavMenuAction::SetDefault(id) => {
                let Some(zone_name) = self.navigation.zone_name_for_id(id) else {
                    return Task::none();
                };
                self.start_default_zone_set(zone_name)
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
                self.operations.confirmation = Some(Confirmation::StopFirewalld);
                return Task::none();
            }
            ZoneViewAction::AddInterface => {
                self.open_context_page(ContextPage::AddInterface);
                return self.start_interfaces_load();
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

        let zone_name = match &self.zones {
            ZoneViewState::Ready(details) => details.name.clone(),
            _ => return Task::none(),
        };

        self.start_zone_item_remove(zone_name, action)
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
                        } => self.start_zone_create(name, description, target),
                        dialogs::Submission::Service { zone, service } => {
                            self.start_service_add(zone, service)
                        }
                        dialogs::Submission::Port {
                            zone,
                            port,
                            protocol,
                        } => self.start_port_add(zone, port, protocol),
                        dialogs::Submission::SourcePort {
                            zone,
                            port,
                            protocol,
                        } => self.start_source_port_add(zone, port, protocol),
                        dialogs::Submission::ForwardPort {
                            zone,
                            port,
                            protocol,
                            to_port,
                            to_addr,
                        } => self.start_forward_port_add(zone, port, protocol, to_port, to_addr),
                        dialogs::Submission::Interface { zone, interface } => {
                            self.start_interface_add(zone, interface)
                        }
                        dialogs::Submission::Source { zone, source } => {
                            self.start_source_add(zone, source)
                        }
                        dialogs::Submission::Icmp { zone, icmp } => self.start_icmp_add(zone, icmp),
                        dialogs::Submission::RichRule { zone, rule } => {
                            self.start_rich_rule_add(zone, rule)
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

    async fn load_zones() -> Result<Vec<String>, BrokerError> {
        let broker = FwdBroker::get().await?;
        broker.get_zones().await
    }

    async fn load_zone_details(zone_name: String) -> Result<ZoneDetails, BrokerError> {
        let broker = FwdBroker::get().await?;
        broker.get_zone_details(&zone_name).await
    }

    async fn load_default_zone() -> Result<String, BrokerError> {
        let broker = FwdBroker::get().await?;
        broker.get_default_zone().await
    }

    async fn load_active_zones() -> Result<HashSet<String>, BrokerError> {
        let broker = FwdBroker::get().await?;
        broker.get_active_zones().await
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

    /// Permanently add a source port to a zone.
    async fn add_source_port(
        zone_name: String,
        port: String,
        protocol: String,
    ) -> Result<(), BrokerError> {
        let broker = FwdBroker::get().await?;
        broker.add_source_port(&zone_name, &port, &protocol).await
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

    fn start_zones_load(&mut self) -> Task<cosmic::Action<Message>> {
        self.navigation.set_loading();
        Task::perform(Self::load_zones(), |result| {
            cosmic::Action::from(Message::Zone(zones::Message::ListLoaded(result)))
        })
    }

    fn start_default_zone_load(&mut self) -> Task<cosmic::Action<Message>> {
        Task::perform(Self::load_default_zone(), |result| {
            cosmic::Action::from(Message::Zone(zones::Message::DefaultLoaded(result)))
        })
    }

    fn start_active_zones_load(&mut self) -> Task<cosmic::Action<Message>> {
        Task::perform(Self::load_active_zones(), |result| {
            cosmic::Action::from(Message::Zone(zones::Message::ActiveLoaded(result)))
        })
    }

    fn start_interfaces_load(&mut self) -> Task<cosmic::Action<Message>> {
        self.update_catalogs(catalogs::Message::LoadInterfaces)
    }

    fn start_services_load(&mut self) -> Task<cosmic::Action<Message>> {
        self.update_catalogs(catalogs::Message::LoadServices)
    }

    fn start_icmp_types_load(&mut self) -> Task<cosmic::Action<Message>> {
        self.update_catalogs(catalogs::Message::LoadIcmpTypes)
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
            cosmic::Action::from(Message::Zone(zones::Message::ItemAdded {
                zone_name: zone_name_for_task.clone(),
                result,
            }))
        })
    }

    fn start_firewalld_status_load(&mut self) -> Task<cosmic::Action<Message>> {
        self.firewalld_status = FirewalldStatus::Loading;
        Task::perform(Self::load_firewalld_status(), |result| {
            cosmic::Action::from(Message::Zone(zones::Message::FirewalldStatusLoaded(result)))
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
            cosmic::Action::from(Message::Zone(zones::Message::ItemAdded {
                zone_name: zone_name_for_task.clone(),
                result,
            }))
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
                cosmic::Action::from(Message::Zone(zones::Message::ItemAdded {
                    zone_name: zone_name_for_task.clone(),
                    result,
                }))
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
            cosmic::Action::from(Message::Zone(zones::Message::DaemonControlFinished(result)))
        })
    }

    fn start_permanent_apply(&mut self) -> Task<cosmic::Action<Message>> {
        self.handle_reconciliation_message(reconciliation::Message::ApplyPermanent)
    }

    fn start_runtime_configuration_persist(&mut self) -> Task<cosmic::Action<Message>> {
        self.handle_reconciliation_message(reconciliation::Message::PersistRuntime)
    }

    fn start_default_zone_set(&mut self, zone_name: String) -> Task<cosmic::Action<Message>> {
        if !self.begin_mutation(fl!("operation-set-default-zone")) {
            return Task::none();
        }
        Task::perform(Self::set_default_zone(zone_name), |result| {
            cosmic::Action::from(Message::Zone(zones::Message::DefaultSet(result)))
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
            cosmic::Action::from(Message::Zone(zones::Message::Created {
                zone_name: zone_name_for_task.clone(),
                result,
            }))
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
            cosmic::Action::from(Message::Zone(zones::Message::ItemAdded {
                zone_name: zone_name_for_task.clone(),
                result,
            }))
        })
    }

    /// Start a permanent source-port mutation.
    fn start_source_port_add(
        &mut self,
        zone_name: String,
        port: String,
        protocol: String,
    ) -> Task<cosmic::Action<Message>> {
        if !self.begin_mutation(fl!("operation-add-source-port")) {
            return Task::none();
        }
        let zone_name_for_task = zone_name.clone();
        Task::perform(
            Self::add_source_port(zone_name, port, protocol),
            move |result| {
                cosmic::Action::from(Message::Zone(zones::Message::ItemAdded {
                    zone_name: zone_name_for_task.clone(),
                    result,
                }))
            },
        )
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
                cosmic::Action::from(Message::Zone(zones::Message::ItemAdded {
                    zone_name: zone_name_for_task.clone(),
                    result,
                }))
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
            cosmic::Action::from(Message::Zone(zones::Message::ItemAdded {
                zone_name: zone_name_for_task.clone(),
                result,
            }))
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
            cosmic::Action::from(Message::Zone(zones::Message::ItemAdded {
                zone_name: zone_name_for_task.clone(),
                result,
            }))
        })
    }

    fn start_icmp_add(&mut self, zone_name: String, icmp: String) -> Task<cosmic::Action<Message>> {
        if !self.begin_mutation(fl!("operation-add-icmp")) {
            return Task::none();
        }
        let zone_name_for_task = zone_name.clone();
        Task::perform(Self::add_icmp_block(zone_name, icmp), move |result| {
            cosmic::Action::from(Message::Zone(zones::Message::ItemAdded {
                zone_name: zone_name_for_task.clone(),
                result,
            }))
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
            cosmic::Action::from(Message::Zone(zones::Message::ItemAdded {
                zone_name: zone_name_for_task.clone(),
                result,
            }))
        })
    }

    fn start_zone_delete(&mut self, zone_name: String) -> Task<cosmic::Action<Message>> {
        if !self.begin_mutation(fl!("operation-delete-zone")) {
            return Task::none();
        }
        let zone_name_for_task = zone_name.clone();
        Task::perform(Self::remove_zone(zone_name), move |result| {
            cosmic::Action::from(Message::Zone(zones::Message::Deleted {
                zone_name: zone_name_for_task.clone(),
                result,
            }))
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
            | ZoneViewAction::Reconciliation(_) => Task::none(),
            ZoneViewAction::AddService
            | ZoneViewAction::AddInterface
            | ZoneViewAction::AddPort { .. }
            | ZoneViewAction::AddSource
            | ZoneViewAction::AddIcmpBlock
            | ZoneViewAction::AddRichRule => Task::none(),
            ZoneViewAction::RemoveService(service) => {
                Task::perform(Self::remove_service(zone_name, service), move |result| {
                    cosmic::Action::from(Message::Zone(zones::Message::ItemRemoved {
                        zone_name: zone_name_for_task.clone(),
                        result,
                    }))
                })
            }
            ZoneViewAction::RemoveInterface(interface) => Task::perform(
                Self::remove_interface(zone_name, interface),
                move |result| {
                    cosmic::Action::from(Message::Zone(zones::Message::ItemRemoved {
                        zone_name: zone_name_for_task.clone(),
                        result,
                    }))
                },
            ),
            ZoneViewAction::RemoveSource(source) => {
                Task::perform(Self::remove_source(zone_name, source), move |result| {
                    cosmic::Action::from(Message::Zone(zones::Message::ItemRemoved {
                        zone_name: zone_name_for_task.clone(),
                        result,
                    }))
                })
            }
            ZoneViewAction::RemovePort { port, protocol } => Task::perform(
                Self::remove_port(zone_name, port, protocol),
                move |result| {
                    cosmic::Action::from(Message::Zone(zones::Message::ItemRemoved {
                        zone_name: zone_name_for_task.clone(),
                        result,
                    }))
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
                    cosmic::Action::from(Message::Zone(zones::Message::ItemRemoved {
                        zone_name: zone_name_for_task.clone(),
                        result,
                    }))
                },
            ),
            ZoneViewAction::RemoveSourcePort { port, protocol } => Task::perform(
                Self::remove_source_port(zone_name, port, protocol),
                move |result| {
                    cosmic::Action::from(Message::Zone(zones::Message::ItemRemoved {
                        zone_name: zone_name_for_task.clone(),
                        result,
                    }))
                },
            ),
            ZoneViewAction::RemoveIcmpBlock(icmp) => {
                Task::perform(Self::remove_icmp_block(zone_name, icmp), move |result| {
                    cosmic::Action::from(Message::Zone(zones::Message::ItemRemoved {
                        zone_name: zone_name_for_task.clone(),
                        result,
                    }))
                })
            }
            ZoneViewAction::RemoveRichRule(rule) => {
                Task::perform(Self::remove_rich_rule(zone_name, rule), move |result| {
                    cosmic::Action::from(Message::Zone(zones::Message::ItemRemoved {
                        zone_name: zone_name_for_task.clone(),
                        result,
                    }))
                })
            }
        }
    }

    fn start_zone_load(&mut self, zone_name: String) -> Task<cosmic::Action<Message>> {
        self.reconciliation
            .selection_changed(Some(zone_name.clone()));
        self.zones = ZoneViewState::Loading {
            zone: zone_name.clone(),
        };

        let zone_name_for_task = zone_name.clone();
        Task::perform(Self::load_zone_details(zone_name), move |result| {
            cosmic::Action::from(Message::Zone(zones::Message::DetailsLoaded {
                zone_name: zone_name_for_task.clone(),
                result: Box::new(result),
            }))
        })
    }

    fn start_zone_reconciliation(&mut self, zone_name: String) -> Task<cosmic::Action<Message>> {
        self.handle_reconciliation_message(reconciliation::Message::Load(zone_name))
    }

    fn start_ipsets_load(&mut self) -> Task<cosmic::Action<Message>> {
        self.update_ipsets(ipsets::Message::LoadList)
    }

    fn start_ipset_delete(&mut self, ipset_name: String) -> Task<cosmic::Action<Message>> {
        self.update_ipsets(ipsets::Message::Delete(ipset_name))
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

/// Return the localized drawer title for the active port operation.
fn port_drawer_title(kind: PortKind) -> String {
    match kind {
        PortKind::Destination => fl!("drawer-title-destination-port"),
        PortKind::Source => fl!("drawer-title-source-port"),
        PortKind::Forward => fl!("drawer-title-forward-port"),
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
