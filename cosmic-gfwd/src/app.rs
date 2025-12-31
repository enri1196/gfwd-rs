// SPDX-License-Identifier: MIT

use crate::config::Config;
use crate::core::{BrokerError, FwdBroker};
use crate::fl;
use crate::models::{IpSetDetails, ZoneDetails};
use crate::ui::{
    drawer_footer, icmp_drawer, interface_drawer, ipset_drawer, port_drawer, rich_rule_drawer,
    source_drawer, target_from_index, view_ipset_content, view_zone_content, DialogKind,
    DialogMessage, DialogState, IpSetViewAction, IpSetViewState, Sidebar, SidebarItem,
    ZoneViewState,
};
use cosmic::app::context_drawer;
use cosmic::cosmic_config::{self, CosmicConfigEntry};
use cosmic::iced::alignment::{Horizontal, Vertical};
use cosmic::iced::{Length, Subscription};
use cosmic::prelude::*;
use cosmic::widget::{self, about::About, menu, nav_bar};
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
    /// State for the IP set view.
    ipset_view: IpSetViewState,
    /// Stores form state for context drawer dialogs.
    dialogs: DialogState,
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
    IpSetAction(IpSetViewAction),
    UpdateConfig(Config),
    ZonesLoaded(Result<Vec<String>, BrokerError>),
    ZoneDetailsLoaded {
        zone_name: String,
        result: Result<ZoneDetails, BrokerError>,
    },
    DefaultZoneLoaded(Result<String, BrokerError>),
    ActiveZonesLoaded(Result<HashSet<String>, BrokerError>),
    ZoneDeleted {
        zone_name: String,
        result: Result<(), BrokerError>,
    },
    IpSetsLoaded(Result<Vec<String>, BrokerError>),
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
            ipset_view: IpSetViewState::default(),
            dialogs: DialogState::default(),
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

        let command = Task::batch(vec![app.update_title(), app.start_zones_load()]);

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
                        menu::Item::Button(
                            fl!("action-add-source"),
                            None,
                            MenuAction::AddSource,
                        ),
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
        let Some(item) = self.sidebar.item_for_id(id) else {
            return None;
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
                            NavMenuAction::OpenZone(id),
                        ),
                        menu::Item::Button(
                            fl!("context-delete-zone"),
                            None,
                            NavMenuAction::DeleteZone(id),
                        ),
                    ],
                ))
            }
            _ => None,
        }
    }

    /// Display a context drawer if the context page is requested.
    fn context_drawer(&self) -> Option<context_drawer::ContextDrawer<'_, Self::Message>> {
        if !self.core.window.show_context {
            return None;
        }

        Some(match self.context_page {
            ContextPage::About => context_drawer::about(
                &self.about,
                |url| Message::LaunchUrl(url.to_string()),
                Message::ToggleContextPage(ContextPage::About),
            ),
            ContextPage::AddZone => context_drawer::context_drawer(
                crate::ui::dialog_drawers::zone_drawer(&self.dialogs.zone),
                DialogMessage::Cancel(DialogKind::Zone),
            )
            .title("Add Zone")
            .footer(drawer_footer(DialogKind::Zone))
            .map(Message::Dialog),
            ContextPage::AddPort => context_drawer::context_drawer(
                port_drawer(&self.dialogs.port),
                DialogMessage::Cancel(DialogKind::Port),
            )
            .title("Add Port")
            .footer(drawer_footer(DialogKind::Port))
            .map(Message::Dialog),
            ContextPage::AddInterface => context_drawer::context_drawer(
                interface_drawer(&self.dialogs.interface),
                DialogMessage::Cancel(DialogKind::Interface),
            )
            .title("Add Interface")
            .footer(drawer_footer(DialogKind::Interface))
            .map(Message::Dialog),
            ContextPage::AddSource => context_drawer::context_drawer(
                source_drawer(&self.dialogs.source),
                DialogMessage::Cancel(DialogKind::Source),
            )
            .title("Add Source")
            .footer(drawer_footer(DialogKind::Source))
            .map(Message::Dialog),
            ContextPage::AddIcmp => context_drawer::context_drawer(
                icmp_drawer(&self.dialogs.icmp),
                DialogMessage::Cancel(DialogKind::Icmp),
            )
            .title("Add ICMP Block")
            .footer(drawer_footer(DialogKind::Icmp))
            .map(Message::Dialog),
            ContextPage::AddRichRule => context_drawer::context_drawer(
                rich_rule_drawer(&self.dialogs.rich_rule),
                DialogMessage::Cancel(DialogKind::RichRule),
            )
            .title("Add Rich Rule")
            .footer(drawer_footer(DialogKind::RichRule))
            .map(Message::Dialog),
            ContextPage::AddIpSet => context_drawer::context_drawer(
                ipset_drawer(&self.dialogs.ipset),
                DialogMessage::Cancel(DialogKind::IpSet),
            )
            .title("Create IP Set")
            .footer(drawer_footer(DialogKind::IpSet))
            .map(Message::Dialog),
        })
    }

    /// Describes the interface based on the current state of the application model.
    ///
    /// Application events will be processed through the view. Any messages emitted by
    /// events received by widgets will be passed to the update method.
    fn view(&self) -> Element<'_, Self::Message> {
        let space_m = cosmic::theme::spacing().space_m;
        let content: Element<_> = match self.sidebar.active_item() {
            Some(SidebarItem::IpSets) => view_ipset_content(&self.ipset_view, Message::IpSetAction),
            _ => view_zone_content(&self.zone_view),
        };

        widget::container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(space_m)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center)
            .into()
    }

    /// Register subscriptions for this application.
    ///
    /// Subscriptions are long-running async tasks running in the background which
    /// emit messages to the application through a channel. They can be dynamically
    /// stopped and started conditionally based on application state, or persist
    /// indefinitely.
    fn subscription(&self) -> Subscription<Self::Message> {
        // Add subscriptions which are always active.
        let subscriptions = vec![
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

        Subscription::batch(subscriptions)
    }

    /// Handles messages emitted by the application and its widgets.
    ///
    /// Tasks may be returned for asynchronous execution of code in the background
    /// on the application's async runtime.
    fn update(&mut self, message: Self::Message) -> Task<cosmic::Action<Self::Message>> {
        match message {
            Message::ToggleContextPage(context_page) => {
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
                }
            }

            Message::Dialog(dialog_message) => {
                return self.handle_dialog_message(dialog_message);
            }

            Message::NavMenuAction(action) => {
                return self.handle_nav_menu_action(action);
            }

            Message::IpSetAction(action) => {
                return self.handle_ipset_action(action);
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
                        }
                        Task::none()
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

                self.zone_view = match result {
                    Ok(details) => ZoneViewState::Ready(details),
                    Err(error) => ZoneViewState::Error {
                        zone: zone_name,
                        message: error.to_string(),
                    },
                };
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
            Message::ZoneDeleted { zone_name, result } => match result {
                Ok(()) => {
                    if matches!(
                        self.sidebar.active_item(),
                        Some(SidebarItem::Zone { name, .. }) if name == &zone_name
                    ) {
                        self.zone_view = ZoneViewState::Empty;
                    }
                    return self.start_zones_load();
                }
                Err(error) => {
                    if matches!(
                        self.sidebar.active_item(),
                        Some(SidebarItem::Zone { name, .. }) if name == &zone_name
                    ) {
                        self.zone_view = ZoneViewState::Error {
                            zone: zone_name,
                            message: error.to_string(),
                        };
                    }
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
                    self.ipset_view.entry_input.clear();
                    self.ipset_view.entry_error = None;
                    return self.start_ipset_details_load(ipset_name);
                }
                Err(error) => {
                    self.ipset_view.entry_error = Some(error.to_string());
                }
            },
            Message::IpSetCreated { ipset_name, result } => match result {
                Ok(()) => {
                    self.ipset_view.selected = Some(ipset_name.clone());
                    self.ipset_view.entry_input.clear();
                    self.ipset_view.entry_error = None;
                    return Task::batch(vec![
                        self.start_ipsets_load(),
                        self.start_ipset_details_load(ipset_name),
                    ]);
                }
                Err(error) => {
                    self.ipset_view.entry_error = Some(error.to_string());
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
    fn reset_dialog_for_context(&mut self, context_page: ContextPage) {
        if let Some(kind) = dialog_kind_for_page(context_page) {
            self.dialogs.reset(kind);
        }
    }

    fn close_context_drawer(&mut self) {
        self.core.window.show_context = false;
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
            NavMenuAction::OpenZone(id) => self.handle_nav_select(id),
            NavMenuAction::DeleteZone(id) => {
                let Some(zone_name) = self.sidebar.zone_name_for_id(id) else {
                    return Task::none();
                };
                self.start_zone_delete(zone_name)
            }
        }
    }

    fn handle_dialog_message(&mut self, message: DialogMessage) -> Task<cosmic::Action<Message>> {
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
            DialogMessage::PortNumberChanged(value) => {
                self.dialogs.port.port = value;
            }
            DialogMessage::PortProtocolSelected(index) => {
                self.dialogs.port.protocol =
                    crate::ui::dialog_drawers::protocol_from_index(index);
            }
            DialogMessage::PortForwardingToggled(value) => {
                self.dialogs.port.forwarding = value;
            }
            DialogMessage::PortForwardDestIpChanged(value) => {
                self.dialogs.port.dest_ip = value;
            }
            DialogMessage::PortForwardDestPortChanged(value) => {
                self.dialogs.port.dest_port = value;
            }
            DialogMessage::InterfaceNameChanged(value) => {
                self.dialogs.interface.interface = value;
            }
            DialogMessage::SourceAddressChanged(value) => {
                self.dialogs.source.source = value;
            }
            DialogMessage::IcmpTypeChanged(value) => {
                self.dialogs.icmp.icmp_type = value;
            }
            DialogMessage::RichRuleChanged(value) => {
                self.dialogs.rich_rule.rule = value;
            }
            DialogMessage::IpSetNameChanged(value) => {
                self.dialogs.ipset.name = value;
            }
            DialogMessage::IpSetTypeSelected(index) => {
                self.dialogs.ipset.ipset_type =
                    crate::ui::dialog_drawers::ipset_from_index(index);
            }
            DialogMessage::IpSetEntriesChanged(value) => {
                self.dialogs.ipset.entries = value;
            }
            DialogMessage::Submit(DialogKind::IpSet) => {
                let name = self.dialogs.ipset.name.trim().to_string();
                let ipset_type = self.dialogs.ipset.ipset_type.trim().to_string();
                let entries = split_ipset_entries(&self.dialogs.ipset.entries);
                self.dialogs.reset(DialogKind::IpSet);
                self.close_context_drawer();
                if name.is_empty() || ipset_type.is_empty() {
                    return Task::none();
                }
                return self.start_ipset_create(name, ipset_type, entries);
            }
            DialogMessage::Submit(kind) | DialogMessage::Cancel(kind) => {
                self.dialogs.reset(kind);
                self.close_context_drawer();
            }
        }
        Task::none()
    }

    fn handle_ipset_action(&mut self, action: IpSetViewAction) -> Task<cosmic::Action<Message>> {
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
                self.ipset_view.entry_error = None;
            }
            IpSetViewAction::AddEntry => {
                let Some(ipset_name) = self.ipset_view.selected.clone() else {
                    return Task::none();
                };
                let entry = self.ipset_view.entry_input.trim();
                if entry.is_empty() {
                    return Task::none();
                }
                return self.start_ipset_entry_add(ipset_name, entry.to_string());
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

    async fn load_default_zone() -> Result<String, BrokerError> {
        let broker = FwdBroker::get().await?;
        broker.get_default_zone().await
    }

    async fn load_active_zones() -> Result<HashSet<String>, BrokerError> {
        let broker = FwdBroker::get().await?;
        broker.get_active_zones().await
    }

    async fn remove_zone(zone_name: String) -> Result<(), BrokerError> {
        let broker = FwdBroker::get().await?;
        broker.remove_zone(&zone_name).await
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

    fn start_zone_delete(&mut self, zone_name: String) -> Task<cosmic::Action<Message>> {
        let zone_name_for_task = zone_name.clone();
        Task::perform(Self::remove_zone(zone_name), move |result| {
            cosmic::Action::from(Message::ZoneDeleted {
                zone_name: zone_name_for_task.clone(),
                result,
            })
        })
    }

    fn start_zone_load(&mut self, zone_name: String) -> Task<cosmic::Action<Message>> {
        self.zone_view = ZoneViewState::Loading {
            zone: zone_name.clone(),
        };

        let zone_name_for_task = zone_name.clone();
        Task::perform(Self::load_zone_details(zone_name), move |result| {
            cosmic::Action::from(Message::ZoneDetailsLoaded {
                zone_name: zone_name_for_task.clone(),
                result,
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
        let ipset_name_for_task = ipset_name.clone();
        Task::perform(Self::add_ipset_entry(ipset_name, entry), move |result| {
            cosmic::Action::from(Message::IpSetEntryAdded {
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
        .split(|c| c == ',' || c == '\n')
        .map(str::trim)
        .filter(|entry| !entry.is_empty())
        .map(|entry| entry.to_string())
        .collect()
}

/// The context page to display in the context drawer.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum ContextPage {
    #[default]
    About,
    AddZone,
    AddPort,
    AddInterface,
    AddSource,
    AddIcmp,
    AddRichRule,
    AddIpSet,
}

fn dialog_kind_for_page(page: ContextPage) -> Option<DialogKind> {
    match page {
        ContextPage::AddZone => Some(DialogKind::Zone),
        ContextPage::AddPort => Some(DialogKind::Port),
        ContextPage::AddInterface => Some(DialogKind::Interface),
        ContextPage::AddSource => Some(DialogKind::Source),
        ContextPage::AddIcmp => Some(DialogKind::Icmp),
        ContextPage::AddRichRule => Some(DialogKind::RichRule),
        ContextPage::AddIpSet => Some(DialogKind::IpSet),
        ContextPage::About => None,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NavMenuAction {
    OpenZone(nav_bar::Id),
    DeleteZone(nav_bar::Id),
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
