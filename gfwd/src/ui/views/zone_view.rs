// Zone view implementation with integrated components

use relm4::actions::{AccelsPlus, ActionGroupName, RelmAction, RelmActionGroup};
use relm4::adw::prelude::*;
use relm4::gtk::glib::{self, LogLevel};
use relm4::prelude::*;

use crate::core::FwdBroker;
use crate::messages::port::PortDialogResponse;
use crate::messages::zone::{ZoneViewRequest, ZoneViewResponse};

use crate::ui::components::{PortItem, PortItemResponse, IcmpItem, IcmpItemResponse, InterfaceItem, InterfaceItemResponse, SourceItem, SourceItemResponse, RichRuleItem, RichRuleItemResponse};
use crate::ui::dialogs::{AddPortDialog, AddIcmpDialog, AddInterfaceDialog, AddSourceDialog, RichRuleDialog};
use crate::messages::icmp::IcmpDialogResponse;
use crate::messages::interface::InterfaceDialogResponse;
use crate::messages::source::SourceDialogResponse;
use crate::messages::rich_rule::RichRuleDialogResponse;

// Service item for the services list
#[derive(Debug, Clone)]
struct ServiceItem {
    name: String,
    enabled: bool,
}

#[derive(Debug)]
enum ServiceItemInput {
    Toggle,
}

#[derive(Debug)]
enum ServiceItemOutput {
    Toggle(String, bool),
}

#[relm4::factory]
impl FactoryComponent for ServiceItem {
    type Init = (String, bool);
    type Input = ServiceItemInput;
    type Output = ServiceItemOutput;
    type CommandOutput = ();
    type ParentWidget = gtk::ListBox;

    view! {
        adw::ActionRow {
            #[watch]
            set_title: &self.name,
            set_subtitle: &format!("Service: {}", self.name),
            set_activatable: true,
            add_prefix = &gtk::Image {
                set_icon_name: Some("preferences-system-network-symbolic"),
                set_pixel_size: 16,
            },
            add_suffix = &gtk::Switch {
                #[watch]
                set_active: self.enabled,
                set_valign: gtk::Align::Center,
                set_vexpand: false,
                connect_state_set[sender] => move |_, _state| {
                    sender.input(ServiceItemInput::Toggle);
                    glib::Propagation::Proceed
                },
            },
            connect_activated[sender] => move |_| {
                sender.input(ServiceItemInput::Toggle);
            },
        }
    }

    fn init_model(
        (name, enabled): Self::Init,
        _index: &DynamicIndex,
        _sender: FactorySender<Self>,
    ) -> Self {
        Self { name, enabled }
    }

    fn update(&mut self, msg: Self::Input, sender: FactorySender<Self>) {
        match msg {
            ServiceItemInput::Toggle => {
                let new_state = !self.enabled;
                self.enabled = new_state;
                sender
                    .output(ServiceItemOutput::Toggle(self.name.clone(), new_state))
                    .unwrap();
            }
        }
    }
}
use crate::utils::constants::{APP_NAME, APP_VERSION};

relm4::new_action_group!(WindowActionGroup, "win");
relm4::new_stateless_action!(DeleteZoneAction, WindowActionGroup, "delete-zone");
relm4::new_stateless_action!(AboutAction, WindowActionGroup, "about");

#[tracker::track]
pub struct ZoneView {
    #[tracker::do_not_track]
    broker: &'static FwdBroker,
    #[tracker::do_not_track]
    port_dialog: AsyncController<AddPortDialog>,
    #[tracker::do_not_track]
    icmp_dialog: AsyncController<AddIcmpDialog>,
    #[tracker::do_not_track]
    interface_dialog: AsyncController<AddInterfaceDialog>,
    #[tracker::do_not_track]
    source_dialog: AsyncController<AddSourceDialog>,
    #[tracker::do_not_track]
    rich_rule_dialog: AsyncController<RichRuleDialog>,
    #[tracker::do_not_track]
    ports: FactoryVecDeque<PortItem>,
    #[tracker::do_not_track]
    services: FactoryVecDeque<ServiceItem>,
    #[tracker::do_not_track]
    icmp_blocks: FactoryVecDeque<IcmpItem>,
    #[tracker::do_not_track]
    interfaces: FactoryVecDeque<InterfaceItem>,
    #[tracker::do_not_track]
    sources: FactoryVecDeque<SourceItem>,
    #[tracker::do_not_track]
    rich_rules: FactoryVecDeque<RichRuleItem>,
    current_zone_name: String,
    firewalld_running: bool,
    // Zone settings
    masquerading: bool,
    icmp_block_inversion: bool,
    target_policy: String,
    // Services
    active_services: Vec<String>,
    available_services: Vec<String>,
    service_filter: String,
    // ICMP
    icmp_block_list: Vec<String>,
    // Interfaces and Sources
    interface_list: Vec<String>,
    source_list: Vec<String>,
    // Rich Rules
    rich_rule_list: Vec<String>,
}

#[relm4::component(async, pub)]
impl AsyncComponent for ZoneView {
    type Init = (String, adw::ApplicationWindow);
    type Input = ZoneViewRequest;
    type Output = ZoneViewResponse;
    type CommandOutput = ();

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,

            adw::HeaderBar {
                set_css_classes: &["flat"],

                pack_start = &gtk::Box {
                    set_spacing: 6,
                    gtk::Button {
                        set_icon_name: "sidebar-show-symbolic",
                        connect_clicked[sender] => move |_| {
                            sender.output(ZoneViewResponse::ToggleSidebar).unwrap();
                        },
                    },
                    gtk::Button {
                        #[track(model.changed(ZoneView::firewalld_running()))]
                        set_icon_name: if model.firewalld_running { "media-playback-stop-symbolic" } else { "media-playback-start-symbolic" },
                        #[track(model.changed(ZoneView::firewalld_running()))]
                        set_tooltip_text: Some(if model.firewalld_running { "Stop Firewalld" } else { "Start Firewalld" }),
                        connect_clicked[sender] => move |_| {
                            sender.input(ZoneViewRequest::ToggleFirewalld);
                        }
                    }
                },

                pack_end = &gtk::MenuButton {
                    set_icon_name: "open-menu-symbolic",
                    set_tooltip_text: Some("Zone actions"),
                    #[wrap(Some)]
                    set_popover = &gtk::PopoverMenu::from_model(Some(&main_menu)) {}
                },
            },

            gtk::ScrolledWindow {
                set_vexpand: true,
                set_hscrollbar_policy: gtk::PolicyType::Never,

                #[wrap(Some)]
                set_child = &adw::Clamp {
                    set_maximum_size: 1200,
                    set_tightening_threshold: 800,

                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_margin_all: 18,
                        set_spacing: 24,

                        // Zone Header
                        adw::PreferencesGroup {
                            add = &gtk::Box {
                                set_orientation: gtk::Orientation::Horizontal,
                                set_spacing: 16,
                                set_margin_all: 16,

                                gtk::Image {
                                    set_icon_name: Some("security-high-symbolic"),
                                    set_pixel_size: 48,
                                    add_css_class: "accent",
                                },

                                gtk::Box {
                                    set_orientation: gtk::Orientation::Vertical,
                                    set_spacing: 4,
                                    set_hexpand: true,
                                    set_valign: gtk::Align::Center,

                                    gtk::Label {
                                        #[track(model.changed(ZoneView::current_zone_name()))]
                                        set_text: &model.current_zone_name,
                                        set_halign: gtk::Align::Start,
                                        add_css_class: "title-1",
                                    },

                                    gtk::Label {
                                        set_text: "Firewall Zone Configuration",
                                        set_halign: gtk::Align::Start,
                                        add_css_class: "subtitle",
                                        add_css_class: "dim-label",
                                    },
                                },

                                gtk::Box {
                                    set_orientation: gtk::Orientation::Horizontal,
                                    set_spacing: 8,
                                    set_valign: gtk::Align::Center,

                                    gtk::Image {
                                        #[track(model.changed(ZoneView::firewalld_running()))]
                                        set_icon_name: Some(if model.firewalld_running { "emblem-ok-symbolic" } else { "dialog-warning-symbolic" }),
                                        set_pixel_size: 16,
                                        set_valign: gtk::Align::Center,
                                    },

                                    gtk::Label {
                                        #[track(model.changed(ZoneView::firewalld_running()))]
                                        set_text: if model.firewalld_running { "Active" } else { "Inactive" },
                                        add_css_class: "caption",
                                        set_valign: gtk::Align::Center,
                                    },
                                },
                            },
                        },

                        // Main Content Grid
                        gtk::Box {
                            set_orientation: gtk::Orientation::Horizontal,
                            set_spacing: 24,
                            set_homogeneous: false,

                            // Left Column - Zone Information
                            gtk::Box {
                                set_orientation: gtk::Orientation::Vertical,
                                set_spacing: 18,
                                set_hexpand: true,

                                // Zone Information
                                adw::PreferencesGroup {
                                    set_title: "Zone Information",
                                    set_description: Some("Basic zone configuration and properties"),

                                    add = &adw::ActionRow {
                                        set_title: "Zone Name",
                                        #[track(model.changed(ZoneView::current_zone_name()))]
                                        set_subtitle: &model.current_zone_name,
                                        add_prefix = &gtk::Image {
                                            set_icon_name: Some("folder-symbolic"),
                                            set_pixel_size: 16,
                                        },
                                    },

                                    add = &adw::ActionRow {
                                        set_title: "Target Policy",
                                        set_subtitle: "Default action for unmatched packets",
                                        add_prefix = &gtk::Image {
                                            set_icon_name: Some("security-medium-symbolic"),
                                            set_pixel_size: 16,
                                        },
                                        add_suffix = &gtk::Label {
                                            #[track(model.changed(ZoneView::target_policy()))]
                                            set_text: &model.target_policy,
                                            add_css_class: "tag",
                                            add_css_class: "success",
                                        },
                                    },

                                    add = &adw::ActionRow {
                                        set_title: "Masquerading",
                                        set_subtitle: "Network address translation",
                                        set_activatable: true,
                                        add_prefix = &gtk::Image {
                                            set_icon_name: Some("network-workgroup-symbolic"),
                                            set_pixel_size: 16,
                                        },
                                        add_suffix = &gtk::Switch {
                                            #[track(model.changed(ZoneView::masquerading()))]
                                            set_active: model.masquerading,
                                            set_valign: gtk::Align::Center,
                                            set_vexpand: false,
                                            connect_state_set[sender] => move |_, _state| {
                                                sender.input(ZoneViewRequest::ToggleMasquerading);
                                                glib::Propagation::Proceed
                                            },
                                        },
                                        connect_activated[sender] => move |_| {
                                            sender.input(ZoneViewRequest::ToggleMasquerading);
                                        },
                                    },

                                    add = &adw::ActionRow {
                                        set_title: "ICMP Block Inversion",
                                        set_subtitle: "Invert ICMP blocking rules",
                                        set_activatable: true,
                                        add_prefix = &gtk::Image {
                                            set_icon_name: Some("network-cellular-signal-none-symbolic"),
                                            set_pixel_size: 16,
                                        },
                                        add_suffix = &gtk::Switch {
                                            #[track(model.changed(ZoneView::icmp_block_inversion()))]
                                            set_active: model.icmp_block_inversion,
                                            set_valign: gtk::Align::Center,
                                            set_vexpand: false,
                                            connect_state_set[sender] => move |_, _state| {
                                                sender.input(ZoneViewRequest::ToggleIcmpBlockInversion);
                                                glib::Propagation::Proceed
                                            },
                                        },
                                        connect_activated[sender] => move |_| {
                                            sender.input(ZoneViewRequest::ToggleIcmpBlockInversion);
                                        },
                                    },
                                },

                                // Interfaces
                                adw::PreferencesGroup {
                                    set_title: "Network Interfaces",
                                    set_description: Some("Interfaces assigned to this zone"),

                                    #[wrap(Some)]
                                    set_header_suffix = &gtk::Button {
                                        set_icon_name: "list-add-symbolic",
                                        set_tooltip_text: Some("Add interface"),
                                        add_css_class: "flat",
                                        connect_clicked[sender] => move |_| {
                                            sender.input(ZoneViewRequest::ShowAddInterfaceDialog);
                                        },
                                    },

                                    #[local_ref]
                                    interfaces_list_box -> gtk::ListBox {
                                        set_selection_mode: gtk::SelectionMode::None,
                                        add_css_class: "boxed-list",
                                        #[track(model.changed(ZoneView::interface_list()))]
                                        set_visible: !model.interface_list.is_empty(),
                                    },

                                    // Show message when no interfaces are assigned
                                    add = &adw::ActionRow {
                                        #[track(model.changed(ZoneView::interface_list()))]
                                        set_visible: model.interface_list.is_empty(),
                                        set_title: "No interfaces assigned",
                                        set_subtitle: "All traffic will use default zone assignment",
                                        add_prefix = &gtk::Image {
                                            set_icon_name: Some("network-wired-symbolic"),
                                            set_pixel_size: 16,
                                            add_css_class: "dim-label",
                                        },
                                    },
                                },

                                // Services
                                adw::PreferencesGroup {
                                    set_title: "Allowed Services",
                                    set_description: Some("Available firewall services (use search to filter)"),

                                    #[wrap(Some)]
                                    set_header_suffix = &gtk::Button {
                                        set_icon_name: "view-refresh-symbolic",
                                        set_tooltip_text: Some("Refresh services"),
                                        add_css_class: "flat",
                                        connect_clicked[sender] => move |_| {
                                            sender.input(ZoneViewRequest::LoadServices);
                                        },
                                    },

                                    // Search entry
                                    add = &adw::EntryRow {
                                        set_title: "Search Services",
                                        set_text: "",
                                        add_prefix = &gtk::Image {
                                            set_icon_name: Some("system-search-symbolic"),
                                            set_pixel_size: 16,
                                        },
                                        connect_changed[sender] => move |entry| {
                                            sender.input(ZoneViewRequest::FilterServices(entry.text().to_string()));
                                        },
                                    },

                                    // Scrollable services list
                                    add = &gtk::ScrolledWindow {
                                        set_policy: (gtk::PolicyType::Never, gtk::PolicyType::Automatic),
                                        set_min_content_height: 200,
                                        set_max_content_height: 400,
                                        set_vexpand: false,

                                        #[local_ref]
                                        #[wrap(Some)]
                                        set_child = services_list_box -> gtk::ListBox,
                                    },

                                    add = &gtk::Button {
                                        set_label: "Manage Services",
                                        add_css_class: "pill",
                                        set_halign: gtk::Align::Center,
                                        set_margin_top: 12,
                                    },
                                },
                            },

                            // Right Column - Ports and Rules
                            gtk::Box {
                                set_orientation: gtk::Orientation::Vertical,
                                set_spacing: 18,
                                set_hexpand: true,

                                // Port Rules
                                adw::PreferencesGroup {
                                    set_title: "Port Rules",
                                    set_description: Some("Custom port access rules and forwarding"),

                                    #[wrap(Some)]
                                    set_header_suffix = &gtk::Button {
                                        set_icon_name: "list-add-symbolic",
                                        set_tooltip_text: Some("Add port rule"),
                                        add_css_class: "flat",
                                        connect_clicked[sender] => move |_| {
                                            sender.input(ZoneViewRequest::ShowAddPortDialog);
                                        },
                                    },

                                    #[local_ref]
                                    ports_list_box -> gtk::ListBox {
                                        set_selection_mode: gtk::SelectionMode::None,
                                        add_css_class: "boxed-list",
                                    },
                                },

                                // Rich Rules
                                adw::PreferencesGroup {
                                    set_title: "Rich Rules",
                                    set_description: Some("Advanced firewall rules with complex conditions"),

                                    #[wrap(Some)]
                                    set_header_suffix = &gtk::Button {
                                        set_icon_name: "list-add-symbolic",
                                        set_tooltip_text: Some("Add rich rule"),
                                        add_css_class: "flat",
                                        connect_clicked[sender] => move |_| {
                                            sender.input(ZoneViewRequest::ShowAddRichRuleDialog);
                                        },
                                    },

                                    #[local_ref]
                                    rich_rules_list_box -> gtk::ListBox {
                                        set_selection_mode: gtk::SelectionMode::None,
                                        add_css_class: "boxed-list",
                                        #[track(model.changed(ZoneView::rich_rule_list()))]
                                        set_visible: !model.rich_rule_list.is_empty(),
                                    },

                                    // Show message when no rich rules are configured
                                    add = &adw::ActionRow {
                                        #[track(model.changed(ZoneView::rich_rule_list()))]
                                        set_visible: model.rich_rule_list.is_empty(),
                                        set_title: "No rich rules configured",
                                        set_subtitle: "Rich rules allow complex firewall configurations",
                                        add_prefix = &gtk::Image {
                                            set_icon_name: Some("applications-system-symbolic"),
                                            set_pixel_size: 16,
                                            add_css_class: "dim-label",
                                        },
                                    },
                                },

                                // Source Addresses
                                adw::PreferencesGroup {
                                    set_title: "Source Addresses",
                                    set_description: Some("IP addresses and networks with access to this zone"),

                                    #[wrap(Some)]
                                    set_header_suffix = &gtk::Button {
                                        set_icon_name: "list-add-symbolic",
                                        set_tooltip_text: Some("Add source address"),
                                        add_css_class: "flat",
                                        connect_clicked[sender] => move |_| {
                                            sender.input(ZoneViewRequest::ShowAddSourceDialog);
                                        },
                                    },

                                    #[local_ref]
                                    sources_list_box -> gtk::ListBox {
                                        set_selection_mode: gtk::SelectionMode::None,
                                        add_css_class: "boxed-list",
                                        #[track(model.changed(ZoneView::source_list()))]
                                        set_visible: !model.source_list.is_empty(),
                                    },

                                    // Show message when no sources are configured
                                    add = &adw::ActionRow {
                                        #[track(model.changed(ZoneView::source_list()))]
                                        set_visible: model.source_list.is_empty(),
                                        set_title: "All sources allowed",
                                        set_subtitle: "No source restrictions configured",
                                        add_prefix = &gtk::Image {
                                            set_icon_name: Some("network-workgroup-symbolic"),
                                            set_pixel_size: 16,
                                            add_css_class: "dim-label",
                                        },
                                    },
                                },

                                // ICMP Blocks
                                adw::PreferencesGroup {
                                    set_title: "ICMP Blocks",
                                    set_description: Some("Blocked ICMP message types for this zone"),

                                    #[wrap(Some)]
                                    set_header_suffix = &gtk::Button {
                                        set_icon_name: "list-add-symbolic",
                                        set_tooltip_text: Some("Add ICMP block"),
                                        add_css_class: "flat",
                                        connect_clicked[sender] => move |_| {
                                            sender.input(ZoneViewRequest::ShowAddIcmpDialog);
                                        },
                                    },

                                    #[local_ref]
                                    icmp_blocks_list_box -> gtk::ListBox {
                                        set_selection_mode: gtk::SelectionMode::None,
                                        add_css_class: "boxed-list",
                                        #[track(model.changed(ZoneView::icmp_block_list()))]
                                        set_visible: !model.icmp_block_list.is_empty(),
                                    },

                                    // Show message when no ICMP blocks are configured
                                    add = &adw::ActionRow {
                                        #[track(model.changed(ZoneView::icmp_block_list()))]
                                        set_visible: model.icmp_block_list.is_empty(),
                                        set_title: "No ICMP blocks configured",
                                        set_subtitle: "All ICMP messages are allowed",
                                        add_prefix = &gtk::Image {
                                            set_icon_name: Some("network-cellular-signal-good-symbolic"),
                                            set_pixel_size: 16,
                                            add_css_class: "success",
                                        },
                                    },
                                },
                            },
                        },
                    },
                },
            },
        }
    }

    menu! {
        main_menu: {
            "Delete Zone" => DeleteZoneAction,
            "About" => AboutAction,
        }
    }

    async fn init(
        (initial_zone_name, _app_window): Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let broker = FwdBroker::get_broker().await;

        let port_dialog =
            AddPortDialog::builder()
                .launch(())
                .forward(sender.input_sender(), |msg| match msg {
                    PortDialogResponse::PortAdded {
                        port,
                        protocol,
                        forwarding,
                    } => {
                        if let Some(forward) = forwarding {
                            ZoneViewRequest::AddForwardPort(
                                port,
                                protocol,
                                forward.to_port,
                                forward.to_addr,
                            )
                        } else {
                            ZoneViewRequest::AddPort(port, protocol)
                        }
                    }
                });

        // Initialize ICMP dialog with available ICMP types
        let icmp_types = broker.get_icmp_types().await.unwrap_or_default();
        let icmp_dialog =
            AddIcmpDialog::builder()
                .launch(icmp_types)
                .forward(sender.input_sender(), |msg| match msg {
                    IcmpDialogResponse::IcmpSelected { name } => {
                        ZoneViewRequest::AddIcmpBlock(name)
                    }
                });

        // Get available interfaces for the dialog
        let available_interfaces = broker.get_interfaces().await.unwrap_or_default();
        
        // Initialize interface dialog
        let interface_dialog =
            AddInterfaceDialog::builder()
                .launch(available_interfaces)
                .forward(sender.input_sender(), |msg| match msg {
                    InterfaceDialogResponse::InterfaceAdded { name } => {
                        ZoneViewRequest::AddInterface(name)
                    }
                });

        // Initialize source dialog
        let source_dialog =
            AddSourceDialog::builder()
                .launch(())
                .forward(sender.input_sender(), |msg| match msg {
                    SourceDialogResponse::SourceAdded { address } => {
                        ZoneViewRequest::AddSource(address)
                    }
                });

        // Initialize rich rule dialog
        let rich_rule_dialog =
            RichRuleDialog::builder()
                .launch(())
                .forward(sender.input_sender(), |msg| match msg {
                    RichRuleDialogResponse::RichRuleCreated { rule_xml } => {
                        ZoneViewRequest::AddRichRule(rule_xml)
                    }
                });

        let ports =
            FactoryVecDeque::builder()
                .launch_default()
                .forward(sender.input_sender(), |msg| match msg {
                    PortItemResponse::RemovePort {
                        port,
                        protocol,
                        forwarding,
                    } => {
                        if let Some(forward) = forwarding {
                            ZoneViewRequest::RemoveForwardPort(
                                port,
                                protocol,
                                forward.to_port,
                                forward.to_addr,
                            )
                        } else {
                            ZoneViewRequest::RemovePort(port, protocol)
                        }
                    }
                });

        let services =
            FactoryVecDeque::builder()
                .launch_default()
                .forward(sender.input_sender(), |msg| match msg {
                    ServiceItemOutput::Toggle(service_name, enabled) => {
                        ZoneViewRequest::ToggleService(service_name, enabled)
                    }
                });

        let icmp_blocks =
            FactoryVecDeque::builder()
                .launch_default()
                .forward(sender.input_sender(), |msg| match msg {
                    IcmpItemResponse::RemoveIcmp { name } => {
                        ZoneViewRequest::RemoveIcmpBlock(name)
                    }
                });

        let interfaces =
            FactoryVecDeque::builder()
                .launch_default()
                .forward(sender.input_sender(), |msg| match msg {
                    InterfaceItemResponse::RemoveInterface { name } => {
                        ZoneViewRequest::RemoveInterface(name)
                    }
                });

        let sources =
            FactoryVecDeque::builder()
                .launch_default()
                .forward(sender.input_sender(), |msg| match msg {
                    SourceItemResponse::RemoveSource { address } => {
                        ZoneViewRequest::RemoveSource(address)
                    }
                });

        let rich_rules =
            FactoryVecDeque::builder()
                .launch_default()
                .forward(sender.input_sender(), |msg| match msg {
                    RichRuleItemResponse::RemoveRichRule { rule_xml } => {
                        ZoneViewRequest::RemoveRichRule(rule_xml)
                    }
                });

        let initial_firewalld_running = broker.is_firewalld_active().await.unwrap_or(false);

        // Load initial zone settings and services
        let initial_zone_name_clone = initial_zone_name.clone();
        let sender_clone = sender.clone();
        relm4::spawn(async move {
            if let Ok(settings) = broker.get_zone_settings(&initial_zone_name_clone).await {
                sender_clone.input(ZoneViewRequest::UpdateZoneSettings(settings));
            }
        });

        // Load available services
        sender.input(ZoneViewRequest::LoadServices);
        
        // Load ICMP blocks
        sender.input(ZoneViewRequest::LoadIcmpTypes);
        
        // Load interfaces and sources
        sender.input(ZoneViewRequest::LoadInterfaces);
        sender.input(ZoneViewRequest::LoadSources);
        
        // Load rich rules
        sender.input(ZoneViewRequest::LoadRichRules);

        let model = ZoneView {
            broker,
            port_dialog,
            icmp_dialog,
            interface_dialog,
            source_dialog,
            rich_rule_dialog,
            ports,
            services,
            icmp_blocks,
            interfaces,
            sources,
            rich_rules,
            current_zone_name: initial_zone_name,
            firewalld_running: initial_firewalld_running,
            masquerading: false,
            icmp_block_inversion: false,
            target_policy: "ACCEPT".to_string(),
            active_services: Vec::new(),
            available_services: Vec::new(),
            service_filter: String::new(),
            icmp_block_list: Vec::new(),
            interface_list: Vec::new(),
            source_list: Vec::new(),
            rich_rule_list: Vec::new(),
            tracker: 0,
        };

        let ports_list_box = model.ports.widget();
        let services_list_box = model.services.widget();
        let icmp_blocks_list_box = model.icmp_blocks.widget();
        let interfaces_list_box = model.interfaces.widget();
        let sources_list_box = model.sources.widget();
        let rich_rules_list_box = model.rich_rules.widget();
        let widgets = view_output!();

        // Set up actions
        let app = relm4::main_application();
        app.set_accelerators_for_action::<DeleteZoneAction>(&["<primary>Delete"]);

        let sender_delete = sender.clone();
        let delete_zone_action: RelmAction<DeleteZoneAction> = {
            let action = RelmAction::new_stateless(move |_| {
                sender_delete.input(ZoneViewRequest::RemoveZone);
            });
            action.set_enabled(true);
            action
        };

        let app_window = _app_window.clone();
        let about_action: RelmAction<AboutAction> = {
            let action = RelmAction::new_stateless(move |_| {
                let about_dialog = adw::AboutDialog::builder()
                    .application_name(APP_NAME)
                    .version(APP_VERSION)
                    .developer_name("GFWD Contributors")
                    .website("https://github.com/enri1196/gfwd-rs")
                    .issue_url("https://github.com/enri1196/gfwd-rs/issues")
                    .comments("A modern GTK4 firewall management application")
                    .license_type(gtk::License::MitX11)
                    .build();
                about_dialog.present(Some(&app_window));
            });
            action.set_enabled(true);
            action
        };

        let mut group = RelmActionGroup::<WindowActionGroup>::new();
        group.add_action(delete_zone_action);
        group.add_action(about_action);
        _app_window.insert_action_group(WindowActionGroup::NAME, Some(&group.into_action_group()));

        AsyncComponentParts { model, widgets }
    }

    async fn update(
        &mut self,
        msg: Self::Input,
        sender: AsyncComponentSender<Self>,
        root: &Self::Root,
    ) {
        self.reset();
        match msg {
            ZoneViewRequest::ShowAddPortDialog => {
                self.port_dialog.widget().present(Some(root));
            }
            ZoneViewRequest::ShowAddIcmpDialog => {
                self.icmp_dialog.widget().present(Some(root));
            }
            ZoneViewRequest::ShowAddInterfaceDialog => {
                self.interface_dialog.widget().present(Some(root));
            }
            ZoneViewRequest::ShowAddSourceDialog => {
                self.source_dialog.widget().present(Some(root));
            }
            ZoneViewRequest::ShowAddRichRuleDialog => {
                self.rich_rule_dialog.widget().present(Some(root));
            }
            ZoneViewRequest::ToggleMasquerading => {
                let new_state = !self.masquerading;
                self.set_masquerading(new_state);

                let broker = self.broker;
                let zone_name = self.current_zone_name.clone();
                relm4::spawn(async move {
                    let result = if new_state {
                        broker.add_masquerade(&zone_name).await
                    } else {
                        broker.remove_masquerade(&zone_name).await
                    };
                    if let Err(e) = result {
                        crate::core::error_handling::async_helpers::log_error(&e, "Failed to toggle masquerading");
                    }
                });
            }
            ZoneViewRequest::ToggleIcmpBlockInversion => {
                let new_state = !self.icmp_block_inversion;
                self.set_icmp_block_inversion(new_state);

                let broker = self.broker;
                let zone_name = self.current_zone_name.clone();
                relm4::spawn(async move {
                    let result = broker.set_icmp_block_inversion(&zone_name, new_state).await;
                    if let Err(e) = result {
                        glib::g_log!(
                            LogLevel::Error,
                            "Failed to toggle ICMP block inversion: {}",
                            e
                        );
                    }
                });
            }
            ZoneViewRequest::AddPort(port, protocol) => {
                let broker = self.broker;
                let zone_name = self.current_zone_name.clone();
                let sender_clone = sender.clone();
                relm4::spawn(async move {
                    let _ = broker
                        .add_port(zone_name.as_str(), port.as_str(), protocol.as_str())
                        .await;
                    if let Ok(settings) = broker.get_zone_settings(&zone_name).await {
                        let _ = sender_clone.input(ZoneViewRequest::UpdateZoneSettings(settings));
                    }
                });
            }
            ZoneViewRequest::SetZoneContent(zone_name) => {
                self.set_current_zone_name(zone_name.clone());
                let sender_clone = sender.clone();
                let broker = self.broker;
                let zone_name = self.current_zone_name.clone();
                relm4::spawn(async move {
                    match broker.get_zone_settings(&zone_name).await {
                        Ok(settings) => {
                            sender_clone.input(ZoneViewRequest::UpdateZoneSettings(settings));
                        }
                        Err(e) => {
                            crate::core::error_handling::async_helpers::log_error(&e, "Failed to update zone content");
                        }
                    }
                });
                
                // Load interfaces and sources for the new zone
                sender.input(ZoneViewRequest::LoadInterfaces);
                sender.input(ZoneViewRequest::LoadSources);
                
                // Load rich rules for the new zone
                sender.input(ZoneViewRequest::LoadRichRules);
            }
            ZoneViewRequest::UpdateZoneSettings(settings) => {
                // Update zone properties
                self.set_masquerading(settings.masquerade);
                self.set_target_policy(settings.target.to_string());
                self.set_active_services(settings.services.clone());

                // Update port list
                let mut ports = self.ports.guard();
                ports.clear();

                // Add regular ports
                for (port, protocol) in &settings.ports {
                    let rule = crate::models::PortRule::new(port.clone(), protocol.clone());
                    ports.push_back(crate::ui::components::PortItem::from(rule));
                }

                // Add forwarded ports
                for (port, protocol, to_port, to_addr) in &settings.forward_ports {
                    let forwarding = crate::models::ForwardingConfig {
                        to_port: to_port.clone(),
                        to_addr: to_addr.clone(),
                    };
                    let rule = crate::models::PortRule::with_forwarding(
                        port.clone(),
                        protocol.clone(),
                        forwarding,
                    );
                    ports.push_back(crate::ui::components::PortItem::from(rule));
                }

                glib::g_log!(
                    LogLevel::Message,
                    "Zone settings updated with {} ports",
                    ports.len()
                );
                drop(ports);

                // Update ICMP blocks
                sender.input(ZoneViewRequest::UpdateIcmpBlocks(settings.icmp_blocks));
                
                // Update interfaces and sources
                sender.input(ZoneViewRequest::UpdateInterfaces(settings.interfaces));
                sender.input(ZoneViewRequest::UpdateSources(settings.sources));
                
                // Update rich rules
                sender.input(ZoneViewRequest::UpdateRichRules(settings.rich_rules));

                // Trigger service loading if we have available services
                if !self.available_services.is_empty() {
                    self.update_services_display();
                } else {
                    // Load services for the first time
                    sender.input(ZoneViewRequest::LoadServices);
                }
            }
            ZoneViewRequest::LoadServices => {
                let broker = self.broker;
                let sender_clone = sender.clone();
                relm4::spawn(async move {
                    match broker.get_services().await {
                        Ok(services) => {
                            // Store available services and update display
                            sender_clone.input(ZoneViewRequest::UpdateAvailableServices(services));
                        }
                        Err(e) => {
                            crate::core::error_handling::async_helpers::log_error(&e, "Failed to load services");
                        }
                    }
                });
            }
            ZoneViewRequest::RemoveZone => {
                // Show confirmation dialog
                let dialog = adw::AlertDialog::builder()
                    .heading("Delete Zone")
                    .body(&format!("Are you sure you want to delete the zone '{}'?\n\nThis action cannot be undone.", self.current_zone_name))
                    .build();

                dialog.add_response("cancel", "Cancel");
                dialog.add_response("delete", "Delete");
                dialog.set_response_appearance("delete", adw::ResponseAppearance::Destructive);
                dialog.set_default_response(Some("cancel"));
                dialog.set_close_response("cancel");

                let broker = self.broker;
                let zone_name = self.current_zone_name.clone();
                let sender_clone = sender.clone();
                dialog.connect_response(None, move |_, response| {
                    if response == "delete" {
                        let broker = broker;
                        let zone_name = zone_name.clone();
                        let sender = sender_clone.clone();
                        relm4::spawn(async move {
                            match broker.remove_zone(zone_name.as_str()).await {
                                Ok(()) => {
                                    glib::g_log!(LogLevel::Info, "Zone deleted successfully");
                                    let _ = sender.output(ZoneViewResponse::RemovedZoneSuccess(
                                        zone_name.clone(),
                                    ));
                                }
                                Err(e) => {
                                    crate::core::error_handling::async_helpers::log_error(&e, "Could not delete zone")
                                }
                            };
                        });
                    }
                });

                if let Some(window) = root.root().and_downcast::<gtk::Window>() {
                    dialog.present(Some(&window));
                } else {
                    dialog.present(gtk::Widget::NONE);
                }
            }
            ZoneViewRequest::SetFirewalldRunning(is_running) => {
                self.set_firewalld_running(is_running);
            }
            ZoneViewRequest::ToggleFirewalld => {
                let desired_start = !self.get_firewalld_running();
                let broker = self.broker;
                let sender = sender.clone();
                self.set_firewalld_running(desired_start);
                relm4::spawn(async move {
                    let result = if desired_start {
                        broker.start_firewalld().await
                    } else {
                        broker.stop_firewalld().await
                    };
                    if let Err(e) = result {
                        crate::core::error_handling::async_helpers::log_error(&e, "Failed to toggle firewalld");
                    }
                    let is_running = broker.is_firewalld_active().await.unwrap_or(false);
                    let _ = sender.input(ZoneViewRequest::SetFirewalldRunning(is_running));
                });
            }
            ZoneViewRequest::AddForwardPort(port, protocol, to_port, to_addr) => {
                let broker = self.broker;
                let zone_name = self.current_zone_name.clone();
                let sender_clone = sender.clone();
                relm4::spawn(async move {
                    let _ = broker
                        .add_forward_port(
                            zone_name.as_str(),
                            port.as_str(),
                            protocol.as_str(),
                            to_port.as_str(),
                            to_addr.as_str(),
                        )
                        .await;
                    if let Ok(settings) = broker.get_zone_settings(&zone_name).await {
                        let _ = sender_clone.input(ZoneViewRequest::UpdateZoneSettings(settings));
                    }
                });
            }
            ZoneViewRequest::RemovePort(port, protocol) => {
                let broker = self.broker;
                let zone_name = self.current_zone_name.clone();
                let sender_clone = sender.clone();
                relm4::spawn(async move {
                    let _ = broker
                        .remove_port(zone_name.as_str(), port.as_str(), protocol.as_str())
                        .await;
                    if let Ok(settings) = broker.get_zone_settings(&zone_name).await {
                        let _ = sender_clone.input(ZoneViewRequest::UpdateZoneSettings(settings));
                    }
                });
            }
            ZoneViewRequest::RemoveForwardPort(port, protocol, to_port, to_addr) => {
                let broker = self.broker;
                let zone_name = self.current_zone_name.clone();
                let sender_clone = sender.clone();
                relm4::spawn(async move {
                    if let Err(err) = broker
                        .remove_forward_port(
                            zone_name.as_str(),
                            port.as_str(),
                            protocol.as_str(),
                            to_port.as_str(),
                            to_addr.as_str(),
                        )
                        .await
                    {
                        crate::core::error_handling::async_helpers::log_error(&err, "Could not remove forward port");
                    } else {
                        if let Ok(settings) = broker.get_zone_settings(&zone_name).await {
                            let _ =
                                sender_clone.input(ZoneViewRequest::UpdateZoneSettings(settings));
                        }
                    }
                });
            }
            ZoneViewRequest::ToggleService(service_name, enabled) => {
                let broker = self.broker;
                let zone_name = self.current_zone_name.clone();
                relm4::spawn(async move {
                    let result = if enabled {
                        broker.add_service(&zone_name, &service_name).await
                    } else {
                        broker.remove_service(&zone_name, &service_name).await
                    };

                    if let Err(e) = result {
                        glib::g_log!(
                            LogLevel::Error,
                            "Failed to toggle service {}: {}",
                            service_name,
                            e
                        );
                    }
                });
            }
            ZoneViewRequest::UpdateAvailableServices(services) => {
                self.set_available_services(services);
                self.update_services_display();
            }
            ZoneViewRequest::FilterServices(filter) => {
                // Only update if the filter actually changed and is reasonable
                if self.service_filter != filter {
                    let filter_len = filter.len();
                    self.set_service_filter(filter);
                    // Only update display if we have services loaded and filter is not too short
                    if !self.available_services.is_empty() && (filter_len == 0 || filter_len >= 2) {
                        self.update_services_display();
                    } else if filter_len == 1 {
                        // Clear the list for single character searches to avoid showing too many results
                        let mut services_list = self.services.guard();
                        services_list.clear();
                    }
                }
            }
            ZoneViewRequest::AddIcmpBlock(icmp_type) => {
                let broker = self.broker;
                let zone_name = self.current_zone_name.clone();
                let sender_clone = sender.clone();
                relm4::spawn(async move {
                    match broker.add_icmp_block(&zone_name, &icmp_type).await {
                        Ok(()) => {
                            // Reload ICMP blocks to update the display
                            sender_clone.input(ZoneViewRequest::LoadIcmpTypes);
                        }
                        Err(e) => {
                            crate::core::error_handling::async_helpers::log_error(&e, "Failed to add ICMP block");
                        }
                    }
                });
            }
            ZoneViewRequest::RemoveIcmpBlock(icmp_type) => {
                let broker = self.broker;
                let zone_name = self.current_zone_name.clone();
                let sender_clone = sender.clone();
                relm4::spawn(async move {
                    match broker.remove_icmp_block(&zone_name, &icmp_type).await {
                        Ok(()) => {
                            // Reload ICMP blocks to update the display
                            sender_clone.input(ZoneViewRequest::LoadIcmpTypes);
                        }
                        Err(e) => {
                            crate::core::error_handling::async_helpers::log_error(&e, "Failed to remove ICMP block");
                        }
                    }
                });
            }
            ZoneViewRequest::LoadIcmpTypes => {
                let broker = self.broker;
                let zone_name = self.current_zone_name.clone();
                let sender_clone = sender.clone();
                relm4::spawn(async move {
                    // Get current ICMP blocks for this zone
                    match broker.get_zone_settings(&zone_name).await {
                        Ok(settings) => {
                            sender_clone.input(ZoneViewRequest::UpdateIcmpBlocks(settings.icmp_blocks));
                        }
                        Err(e) => {
                            crate::core::error_handling::async_helpers::log_error(&e, "Failed to load ICMP blocks");
                        }
                    }
                });
            }
            ZoneViewRequest::UpdateIcmpBlocks(icmp_blocks) => {
                self.set_icmp_block_list(icmp_blocks.clone());
                
                // Update the ICMP blocks display
                let mut icmp_list = self.icmp_blocks.guard();
                icmp_list.clear();
                
                for icmp_type in icmp_blocks {
                    icmp_list.push_back(IcmpItem::from(icmp_type));
                }
                
                glib::g_log!(
                    LogLevel::Debug,
                    "ICMP blocks updated: {} blocks",
                    icmp_list.len()
                );
            }
            ZoneViewRequest::AddInterface(interface_name) => {
                let broker = self.broker;
                let zone_name = self.current_zone_name.clone();
                let sender_clone = sender.clone();
                relm4::spawn(async move {
                    match broker.add_interface_to_zone(&zone_name, &interface_name).await {
                        Ok(()) => {
                            // Reload interfaces to update the display
                            sender_clone.input(ZoneViewRequest::LoadInterfaces);
                        }
                        Err(e) => {
                            crate::core::error_handling::async_helpers::log_error(&e, "Failed to add interface");
                        }
                    }
                });
            }
            ZoneViewRequest::RemoveInterface(interface_name) => {
                let broker = self.broker;
                let zone_name = self.current_zone_name.clone();
                let sender_clone = sender.clone();
                relm4::spawn(async move {
                    match broker.remove_interface_from_zone(&zone_name, &interface_name).await {
                        Ok(()) => {
                            // Reload interfaces to update the display
                            sender_clone.input(ZoneViewRequest::LoadInterfaces);
                        }
                        Err(e) => {
                            crate::core::error_handling::async_helpers::log_error(&e, "Failed to remove interface");
                        }
                    }
                });
            }
            ZoneViewRequest::LoadInterfaces => {
                let broker = self.broker;
                let zone_name = self.current_zone_name.clone();
                let sender_clone = sender.clone();
                relm4::spawn(async move {
                    // Get current interfaces for this zone
                    match broker.get_zone_settings(&zone_name).await {
                        Ok(settings) => {
                            sender_clone.input(ZoneViewRequest::UpdateInterfaces(settings.interfaces));
                        }
                        Err(e) => {
                            crate::core::error_handling::async_helpers::log_error(&e, "Failed to load interfaces");
                        }
                    }
                });
            }
            ZoneViewRequest::UpdateInterfaces(interfaces) => {
                self.set_interface_list(interfaces.clone());
                
                // Update the interfaces display
                let mut interface_list = self.interfaces.guard();
                interface_list.clear();
                
                for interface_name in interfaces {
                    interface_list.push_back(InterfaceItem::from(interface_name));
                }
                
                glib::g_log!(
                    LogLevel::Debug,
                    "Interfaces updated: {} interfaces",
                    interface_list.len()
                );
            }
            ZoneViewRequest::AddSource(source_address) => {
                let broker = self.broker;
                let zone_name = self.current_zone_name.clone();
                let sender_clone = sender.clone();
                relm4::spawn(async move {
                    match broker.add_source_to_zone(&zone_name, &source_address).await {
                        Ok(()) => {
                            // Reload sources to update the display
                            sender_clone.input(ZoneViewRequest::LoadSources);
                        }
                        Err(e) => {
                            crate::core::error_handling::async_helpers::log_error(&e, "Failed to add source");
                        }
                    }
                });
            }
            ZoneViewRequest::RemoveSource(source_address) => {
                let broker = self.broker;
                let zone_name = self.current_zone_name.clone();
                let sender_clone = sender.clone();
                relm4::spawn(async move {
                    match broker.remove_source_from_zone(&zone_name, &source_address).await {
                        Ok(()) => {
                            // Reload sources to update the display
                            sender_clone.input(ZoneViewRequest::LoadSources);
                        }
                        Err(e) => {
                            crate::core::error_handling::async_helpers::log_error(&e, "Failed to remove source");
                        }
                    }
                });
            }
            ZoneViewRequest::LoadSources => {
                let broker = self.broker;
                let zone_name = self.current_zone_name.clone();
                let sender_clone = sender.clone();
                relm4::spawn(async move {
                    // Get current sources for this zone
                    match broker.get_zone_settings(&zone_name).await {
                        Ok(settings) => {
                            sender_clone.input(ZoneViewRequest::UpdateSources(settings.sources));
                        }
                        Err(e) => {
                            crate::core::error_handling::async_helpers::log_error(&e, "Failed to load sources");
                        }
                    }
                });
            }
            ZoneViewRequest::UpdateSources(sources) => {
                self.set_source_list(sources.clone());
                
                // Update the sources display
                let mut source_list = self.sources.guard();
                source_list.clear();
                
                for source_address in sources {
                    source_list.push_back(SourceItem::from(source_address));
                }
                
                glib::g_log!(
                    LogLevel::Debug,
                    "Sources updated: {} sources",
                    source_list.len()
                );
            }
            ZoneViewRequest::AddRichRule(rule_xml) => {
                let broker = self.broker;
                let zone_name = self.current_zone_name.clone();
                let sender_clone = sender.clone();
                relm4::spawn(async move {
                    match broker.add_rich_rule(&zone_name, &rule_xml).await {
                        Ok(()) => {
                            // Reload rich rules to update the display
                            sender_clone.input(ZoneViewRequest::LoadRichRules);
                        }
                        Err(e) => {
                            crate::core::error_handling::async_helpers::log_error(&e, "Failed to add rich rule");
                        }
                    }
                });
            }
            ZoneViewRequest::RemoveRichRule(rule_xml) => {
                let broker = self.broker;
                let zone_name = self.current_zone_name.clone();
                let sender_clone = sender.clone();
                relm4::spawn(async move {
                    match broker.remove_rich_rule(&zone_name, &rule_xml).await {
                        Ok(()) => {
                            // Reload rich rules to update the display
                            sender_clone.input(ZoneViewRequest::LoadRichRules);
                        }
                        Err(e) => {
                            crate::core::error_handling::async_helpers::log_error(&e, "Failed to remove rich rule");
                        }
                    }
                });
            }
            ZoneViewRequest::LoadRichRules => {
                let broker = self.broker;
                let zone_name = self.current_zone_name.clone();
                let sender_clone = sender.clone();
                relm4::spawn(async move {
                    // Get current rich rules for this zone
                    match broker.get_rich_rules(&zone_name).await {
                        Ok(rich_rules) => {
                            sender_clone.input(ZoneViewRequest::UpdateRichRules(rich_rules));
                        }
                        Err(e) => {
                            crate::core::error_handling::async_helpers::log_error(&e, "Failed to load rich rules");
                        }
                    }
                });
            }
            ZoneViewRequest::UpdateRichRules(rich_rules) => {
                self.set_rich_rule_list(rich_rules.clone());
                
                // Update the rich rules display
                let mut rich_rule_list = self.rich_rules.guard();
                rich_rule_list.clear();
                
                for rule_xml in rich_rules {
                    rich_rule_list.push_back(rule_xml);
                }
                
                glib::g_log!(
                    LogLevel::Debug,
                    "Rich rules updated: {} rules",
                    rich_rule_list.len()
                );
            }
        }
    }
}

impl ZoneView {
    fn update_services_display(&mut self) {
        // Early return if no services available
        if self.available_services.is_empty() {
            return;
        }

        let mut services_list = self.services.guard();
        services_list.clear();

        // Filter services based on search term with a limit to prevent UI freezing
        let filter_lower = self.service_filter.to_lowercase();
        let max_display_services = if self.service_filter.is_empty() { 200 } else { 50 };
        
        let mut count = 0;
        for service in &self.available_services {
            if count >= max_display_services {
                break;
            }
            
            if self.service_filter.is_empty() || service.to_lowercase().contains(&filter_lower) {
                let is_enabled = self.active_services.contains(service);
                services_list.push_back((service.clone(), is_enabled));
                count += 1;
            }
        }

        // Show a message if we hit the limit
        if count >= max_display_services && !self.service_filter.is_empty() {
            glib::g_log!(
                LogLevel::Info,
                "Showing first {} matching services. Refine search to see more.",
                max_display_services
            );
        }
    }
}
