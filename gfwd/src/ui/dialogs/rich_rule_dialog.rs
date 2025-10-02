use relm4::adw::prelude::*;
use relm4::gtk::glib;
use relm4::prelude::*;

use crate::core::validation::{validate_source_address, validate_port, validate_protocol, validate_rich_rule_logic};
use crate::messages::rich_rule::{RichRuleDialogRequest, RichRuleDialogResponse};
use crate::models::{RichRule, RichRuleAction};

#[tracker::track]
#[derive(Debug)]
pub struct RichRuleDialog {
    // Rule components
    family: String,
    
    // Source configuration
    source_address: String,
    source_invert: bool,
    source_error: Option<String>,
    source_valid: bool,
    
    // Destination configuration
    destination_address: String,
    destination_invert: bool,
    destination_error: Option<String>,
    destination_valid: bool,
    
    // Rule type (service, port, or protocol)
    rule_type: String,
    
    // Service configuration
    service_name: String,
    service_error: Option<String>,
    service_valid: bool,
    
    // Port configuration
    port_number: String,
    port_protocol: String,
    port_error: Option<String>,
    port_valid: bool,
    
    // Protocol configuration
    protocol_name: String,
    protocol_error: Option<String>,
    protocol_valid: bool,
    
    // Action configuration
    action_type: String,
    mark_value: String,
    reject_type: String,
    action_error: Option<String>,
    action_valid: bool,
    
    // Overall validation
    is_valid: bool,
}

#[relm4::component(async, pub)]
impl SimpleAsyncComponent for RichRuleDialog {
    type Init = ();
    type Input = RichRuleDialogRequest;
    type Output = RichRuleDialogResponse;

    view! {
        dialog = adw::Dialog {
            set_title: "Create Rich Rule",
            set_content_width: 500,
            set_content_height: 700,

            #[wrap(Some)]
            set_child = &adw::ToolbarView {
                add_top_bar = &adw::HeaderBar {
                    set_show_end_title_buttons: false,
                    add_css_class: "flat",

                    #[wrap(Some)]
                    set_title_widget = &adw::WindowTitle {
                        set_title: "Create Rich Rule",
                        set_subtitle: "Build advanced firewall rule with conditions and actions",
                    },

                    pack_start = &gtk::Button {
                        set_label: "Cancel",
                        set_valign: gtk::Align::Center,
                        set_vexpand: false,
                        connect_clicked[sender, dialog] => move |_| {
                            sender.input(RichRuleDialogRequest::Cancel);
                            dialog.close();
                        },
                    },

                    pack_end = &gtk::Button {
                        set_label: "Create Rule",
                        add_css_class: "suggested-action",
                        set_valign: gtk::Align::Center,
                        set_vexpand: false,
                        #[track(model.changed(RichRuleDialog::is_valid()))]
                        set_sensitive: model.is_valid,
                        connect_clicked => RichRuleDialogRequest::Create,
                    },
                },

                #[wrap(Some)]
                set_content = &gtk::ScrolledWindow {
                    set_hscrollbar_policy: gtk::PolicyType::Never,
                    set_vscrollbar_policy: gtk::PolicyType::Automatic,

                    adw::Clamp {
                        set_maximum_size: 450,
                        set_tightening_threshold: 400,

                        adw::PreferencesPage {
                            set_icon_name: Some("security-high-symbolic"),
                            set_title: "Rich Rule Configuration",
                            set_description: "Configure advanced firewall rule with multiple conditions and actions",

                            // Basic Configuration
                            add = &adw::PreferencesGroup {
                                set_title: "Basic Settings",
                                set_description: Some("Configure IP family and rule scope"),

                                add = &adw::ComboRow {
                                    set_title: "IP Family",
                                    set_subtitle: "Select IP version for this rule",
                                    set_model: Some(&gtk::StringList::new(&[
                                        "Any (IPv4 and IPv6)",
                                        "IPv4 only",
                                        "IPv6 only",
                                    ])),
                                    set_selected: 0, // Default to "Any"
                                    connect_selected_notify[sender] => move |combo| {
                                        if let Some(selected) = combo.selected_item() {
                                            if let Some(string_obj) = selected.downcast_ref::<gtk::StringObject>() {
                                                let family = match string_obj.string().as_str() {
                                                    "IPv4 only" => "ipv4".to_string(),
                                                    "IPv6 only" => "ipv6".to_string(),
                                                    _ => String::new(), // Any/default
                                                };
                                                sender.input(RichRuleDialogRequest::SetFamily(family));
                                            }
                                        }
                                    },

                                    add_prefix = &gtk::Image {
                                        set_icon_name: Some("network-wired-symbolic"),
                                        set_pixel_size: 16,
                                    },
                                },
                            },

                            // Source Configuration
                            add = &adw::PreferencesGroup {
                                set_title: "Source Address (Optional)",
                                set_description: Some("Specify source IP address or network range"),

                                add = &adw::EntryRow {
                                    set_title: "Source Address",
                                    set_text: &model.source_address,
                                    connect_changed[sender] => move |entry| {
                                        sender.input(RichRuleDialogRequest::SetSourceAddress(entry.text().to_string()));
                                    },
                                    connect_apply[sender] => move |_| {
                                        sender.input(RichRuleDialogRequest::ValidateSource);
                                    },

                                    add_prefix = &gtk::Image {
                                        set_icon_name: Some("network-server-symbolic"),
                                        set_pixel_size: 16,
                                    },
                                },

                                add = &adw::ActionRow {
                                    set_title: "Invert Source Match",
                                    set_subtitle: "Match all addresses EXCEPT the specified source",

                                    add_suffix = &gtk::Switch {
                                        #[track(model.changed(RichRuleDialog::source_invert()))]
                                        set_active: model.source_invert,
                                        set_valign: gtk::Align::Center,
                                        set_vexpand: false,
                                        connect_state_set[sender] => move |_, state| {
                                            sender.input(RichRuleDialogRequest::SetSourceInvert(state));
                                            glib::Propagation::Proceed
                                        },
                                    },

                                    add_prefix = &gtk::Image {
                                        set_icon_name: Some("object-flip-horizontal-symbolic"),
                                        set_pixel_size: 16,
                                    },
                                },

                                // Source validation error
                                add = &adw::ActionRow {
                                    #[track(model.changed(RichRuleDialog::source_error()))]
                                    set_visible: model.source_error.is_some(),
                                    #[track(model.changed(RichRuleDialog::source_error()))]
                                    set_title: &model.source_error.as_deref().unwrap_or(""),
                                    add_css_class: "error",

                                    add_prefix = &gtk::Image {
                                        set_icon_name: Some("dialog-warning-symbolic"),
                                        add_css_class: "error",
                                    },
                                },
                            },

                            // Destination Configuration
                            add = &adw::PreferencesGroup {
                                set_title: "Destination Address (Optional)",
                                set_description: Some("Specify destination IP address or network range"),

                                add = &adw::EntryRow {
                                    set_title: "Destination Address",
                                    set_text: &model.destination_address,
                                    connect_changed[sender] => move |entry| {
                                        sender.input(RichRuleDialogRequest::SetDestinationAddress(entry.text().to_string()));
                                    },
                                    connect_apply[sender] => move |_| {
                                        sender.input(RichRuleDialogRequest::ValidateDestination);
                                    },

                                    add_prefix = &gtk::Image {
                                        set_icon_name: Some("network-server-symbolic"),
                                        set_pixel_size: 16,
                                    },
                                },

                                add = &adw::ActionRow {
                                    set_title: "Invert Destination Match",
                                    set_subtitle: "Match all addresses EXCEPT the specified destination",

                                    add_suffix = &gtk::Switch {
                                        #[track(model.changed(RichRuleDialog::destination_invert()))]
                                        set_active: model.destination_invert,
                                        set_valign: gtk::Align::Center,
                                        set_vexpand: false,
                                        connect_state_set[sender] => move |_, state| {
                                            sender.input(RichRuleDialogRequest::SetDestinationInvert(state));
                                            glib::Propagation::Proceed
                                        },
                                    },

                                    add_prefix = &gtk::Image {
                                        set_icon_name: Some("object-flip-horizontal-symbolic"),
                                        set_pixel_size: 16,
                                    },
                                },

                                // Destination validation error
                                add = &adw::ActionRow {
                                    #[track(model.changed(RichRuleDialog::destination_error()))]
                                    set_visible: model.destination_error.is_some(),
                                    #[track(model.changed(RichRuleDialog::destination_error()))]
                                    set_title: &model.destination_error.as_deref().unwrap_or(""),
                                    add_css_class: "error",

                                    add_prefix = &gtk::Image {
                                        set_icon_name: Some("dialog-warning-symbolic"),
                                        add_css_class: "error",
                                    },
                                },
                            },

                            // Service/Port/Protocol Selection
                            add = &adw::PreferencesGroup {
                                set_title: "Traffic Specification",
                                set_description: Some("Specify what traffic this rule applies to"),

                                add = &adw::ComboRow {
                                    set_title: "Rule Type",
                                    set_subtitle: "Choose how to specify the traffic",
                                    set_model: Some(&gtk::StringList::new(&[
                                        "Any Traffic",
                                        "Service",
                                        "Port and Protocol",
                                        "Protocol Only",
                                    ])),
                                    set_selected: 0, // Default to "Any Traffic"
                                    connect_selected_notify[sender] => move |combo| {
                                        if let Some(selected) = combo.selected_item() {
                                            if let Some(string_obj) = selected.downcast_ref::<gtk::StringObject>() {
                                                let rule_type = string_obj.string().to_string();
                                                sender.input(RichRuleDialogRequest::SetRuleType(rule_type));
                                            }
                                        }
                                    },

                                    add_prefix = &gtk::Image {
                                        set_icon_name: Some("preferences-system-symbolic"),
                                        set_pixel_size: 16,
                                    },
                                },

                                // Service configuration (visible when rule_type is "Service")
                                add = &adw::EntryRow {
                                    #[track(model.changed(RichRuleDialog::rule_type()))]
                                    set_visible: model.rule_type == "Service",
                                    set_title: "Service Name",
                                    set_text: &model.service_name,
                                    connect_changed[sender] => move |entry| {
                                        sender.input(RichRuleDialogRequest::SetService(entry.text().to_string()));
                                    },
                                    connect_apply[sender] => move |_| {
                                        sender.input(RichRuleDialogRequest::ValidateService);
                                    },

                                    add_prefix = &gtk::Image {
                                        set_icon_name: Some("applications-system-symbolic"),
                                        set_pixel_size: 16,
                                    },
                                },

                                // Service validation error
                                add = &adw::ActionRow {
                                    #[track(model.changed(RichRuleDialog::service_error()) | model.changed(RichRuleDialog::rule_type()))]
                                    set_visible: model.service_error.is_some() && model.rule_type == "Service",
                                    #[track(model.changed(RichRuleDialog::service_error()))]
                                    set_title: &model.service_error.as_deref().unwrap_or(""),
                                    add_css_class: "error",

                                    add_prefix = &gtk::Image {
                                        set_icon_name: Some("dialog-warning-symbolic"),
                                        add_css_class: "error",
                                    },
                                },

                                // Port configuration (visible when rule_type is "Port and Protocol")
                                add = &adw::EntryRow {
                                    #[track(model.changed(RichRuleDialog::rule_type()))]
                                    set_visible: model.rule_type == "Port and Protocol",
                                    set_title: "Port Number",
                                    set_text: &model.port_number,
                                    connect_changed[sender] => move |entry| {
                                        sender.input(RichRuleDialogRequest::SetPortNumber(entry.text().to_string()));
                                    },
                                    connect_apply[sender] => move |_| {
                                        sender.input(RichRuleDialogRequest::ValidatePort);
                                    },

                                    add_prefix = &gtk::Image {
                                        set_icon_name: Some("network-wired-symbolic"),
                                        set_pixel_size: 16,
                                    },
                                },

                                add = &adw::ComboRow {
                                    #[track(model.changed(RichRuleDialog::rule_type()))]
                                    set_visible: model.rule_type == "Port and Protocol",
                                    set_title: "Port Protocol",
                                    set_subtitle: "Select protocol for the port",
                                    set_model: Some(&gtk::StringList::new(&[
                                        "tcp",
                                        "udp",
                                        "sctp",
                                        "dccp",
                                    ])),
                                    set_selected: 0, // Default to "tcp"
                                    connect_selected_notify[sender] => move |combo| {
                                        if let Some(selected) = combo.selected_item() {
                                            if let Some(string_obj) = selected.downcast_ref::<gtk::StringObject>() {
                                                sender.input(RichRuleDialogRequest::SetPortProtocol(string_obj.string().to_string()));
                                            }
                                        }
                                    },

                                    add_prefix = &gtk::Image {
                                        set_icon_name: Some("preferences-system-symbolic"),
                                        set_pixel_size: 16,
                                    },
                                },

                                // Port validation error
                                add = &adw::ActionRow {
                                    #[track(model.changed(RichRuleDialog::port_error()) | model.changed(RichRuleDialog::rule_type()))]
                                    set_visible: model.port_error.is_some() && model.rule_type == "Port and Protocol",
                                    #[track(model.changed(RichRuleDialog::port_error()))]
                                    set_title: &model.port_error.as_deref().unwrap_or(""),
                                    add_css_class: "error",

                                    add_prefix = &gtk::Image {
                                        set_icon_name: Some("dialog-warning-symbolic"),
                                        add_css_class: "error",
                                    },
                                },

                                // Protocol configuration (visible when rule_type is "Protocol Only")
                                add = &adw::ComboRow {
                                    #[track(model.changed(RichRuleDialog::rule_type()))]
                                    set_visible: model.rule_type == "Protocol Only",
                                    set_title: "Protocol",
                                    set_subtitle: "Select network protocol",
                                    set_model: Some(&gtk::StringList::new(&[
                                        "tcp",
                                        "udp",
                                        "icmp",
                                        "ipv6-icmp",
                                        "esp",
                                        "ah",
                                        "sctp",
                                        "mh",
                                    ])),
                                    set_selected: 0, // Default to "tcp"
                                    connect_selected_notify[sender] => move |combo| {
                                        if let Some(selected) = combo.selected_item() {
                                            if let Some(string_obj) = selected.downcast_ref::<gtk::StringObject>() {
                                                sender.input(RichRuleDialogRequest::SetProtocol(string_obj.string().to_string()));
                                            }
                                        }
                                    },

                                    add_prefix = &gtk::Image {
                                        set_icon_name: Some("preferences-system-symbolic"),
                                        set_pixel_size: 16,
                                    },
                                },

                                // Protocol validation error
                                add = &adw::ActionRow {
                                    #[track(model.changed(RichRuleDialog::protocol_error()) | model.changed(RichRuleDialog::rule_type()))]
                                    set_visible: model.protocol_error.is_some() && model.rule_type == "Protocol Only",
                                    #[track(model.changed(RichRuleDialog::protocol_error()))]
                                    set_title: &model.protocol_error.as_deref().unwrap_or(""),
                                    add_css_class: "error",

                                    add_prefix = &gtk::Image {
                                        set_icon_name: Some("dialog-warning-symbolic"),
                                        add_css_class: "error",
                                    },
                                },
                            },

                            // Action Configuration
                            add = &adw::PreferencesGroup {
                                set_title: "Action",
                                set_description: Some("Choose what to do with matching traffic"),

                                add = &adw::ComboRow {
                                    set_title: "Action Type",
                                    set_subtitle: "Select the action for matching traffic",
                                    set_model: Some(&gtk::StringList::new(&[
                                        "Accept",
                                        "Reject",
                                        "Drop",
                                        "Mark",
                                    ])),
                                    set_selected: 0, // Default to "Accept"
                                    connect_selected_notify[sender] => move |combo| {
                                        if let Some(selected) = combo.selected_item() {
                                            if let Some(string_obj) = selected.downcast_ref::<gtk::StringObject>() {
                                                sender.input(RichRuleDialogRequest::SetAction(string_obj.string().to_string()));
                                            }
                                        }
                                    },

                                    add_prefix = &gtk::Image {
                                        set_icon_name: Some("security-high-symbolic"),
                                        set_pixel_size: 16,
                                    },
                                },

                                // Reject type configuration (visible when action is "Reject")
                                add = &adw::ComboRow {
                                    #[track(model.changed(RichRuleDialog::action_type()))]
                                    set_visible: model.action_type == "Reject",
                                    set_title: "Reject Type",
                                    set_subtitle: "How to reject the connection",
                                    set_model: Some(&gtk::StringList::new(&[
                                        "Default",
                                        "icmp-host-prohibited",
                                        "icmp-port-unreachable",
                                        "icmp-admin-prohibited",
                                        "tcp-reset",
                                    ])),
                                    set_selected: 0, // Default to "Default"
                                    connect_selected_notify[sender] => move |combo| {
                                        if let Some(selected) = combo.selected_item() {
                                            if let Some(string_obj) = selected.downcast_ref::<gtk::StringObject>() {
                                                let reject_type = if string_obj.string() == "Default" {
                                                    String::new()
                                                } else {
                                                    string_obj.string().to_string()
                                                };
                                                sender.input(RichRuleDialogRequest::SetRejectType(reject_type));
                                            }
                                        }
                                    },

                                    add_prefix = &gtk::Image {
                                        set_icon_name: Some("dialog-error-symbolic"),
                                        set_pixel_size: 16,
                                    },
                                },

                                // Mark value configuration (visible when action is "Mark")
                                add = &adw::EntryRow {
                                    #[track(model.changed(RichRuleDialog::action_type()))]
                                    set_visible: model.action_type == "Mark",
                                    set_title: "Mark Value",
                                    set_text: &model.mark_value,
                                    connect_changed[sender] => move |entry| {
                                        sender.input(RichRuleDialogRequest::SetMarkValue(entry.text().to_string()));
                                    },
                                    connect_apply[sender] => move |_| {
                                        sender.input(RichRuleDialogRequest::ValidateAction);
                                    },

                                    add_prefix = &gtk::Image {
                                        set_icon_name: Some("emblem-important-symbolic"),
                                        set_pixel_size: 16,
                                    },
                                },

                                // Action validation error
                                add = &adw::ActionRow {
                                    #[track(model.changed(RichRuleDialog::action_error()))]
                                    set_visible: model.action_error.is_some(),
                                    #[track(model.changed(RichRuleDialog::action_error()))]
                                    set_title: &model.action_error.as_deref().unwrap_or(""),
                                    add_css_class: "error",

                                    add_prefix = &gtk::Image {
                                        set_icon_name: Some("dialog-warning-symbolic"),
                                        add_css_class: "error",
                                    },
                                },
                            },

                            // Rule Preview
                            add = &adw::PreferencesGroup {
                                set_title: "Rule Preview",
                                set_description: Some("Preview of the generated rich rule"),

                                add = &gtk::Box {
                                    set_orientation: gtk::Orientation::Horizontal,
                                    set_spacing: 12,
                                    set_margin_all: 12,

                                    gtk::Image {
                                        #[track(model.changed(RichRuleDialog::is_valid()))]
                                        set_icon_name: Some(if model.is_valid { "emblem-ok-symbolic" } else { "dialog-warning-symbolic" }),
                                        set_pixel_size: 16,
                                        set_valign: gtk::Align::Start,
                                        set_margin_top: 2,
                                    },

                                    gtk::Box {
                                        set_orientation: gtk::Orientation::Vertical,
                                        set_spacing: 4,
                                        set_hexpand: true,

                                        gtk::Label {
                                            set_text: "Generated Rule",
                                            set_halign: gtk::Align::Start,
                                            add_css_class: "heading",
                                        },

                                        gtk::Label {
                                            #[track(model.changed(RichRuleDialog::family()) | model.changed(RichRuleDialog::source_address()) | model.changed(RichRuleDialog::source_invert()) | model.changed(RichRuleDialog::destination_address()) | model.changed(RichRuleDialog::destination_invert()) | model.changed(RichRuleDialog::rule_type()) | model.changed(RichRuleDialog::service_name()) | model.changed(RichRuleDialog::port_number()) | model.changed(RichRuleDialog::port_protocol()) | model.changed(RichRuleDialog::protocol_name()) | model.changed(RichRuleDialog::action_type()) | model.changed(RichRuleDialog::mark_value()) | model.changed(RichRuleDialog::reject_type()) | model.changed(RichRuleDialog::is_valid()))]
                                            set_text: &{
                                                // Always generate preview XML
                                                let rule = model.build_rich_rule();
                                                rule.to_xml()
                                            },
                                            set_halign: gtk::Align::Start,
                                            set_selectable: true,
                                            set_wrap: true,
                                            set_wrap_mode: gtk::pango::WrapMode::WordChar,
                                            add_css_class: "monospace",
                                            add_css_class: "caption",
                                        },
                                    },
                                },
                            },
                        }
                    }
                }
            }
        }
    }

    async fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let model = Self {
            family: String::new(),
            source_address: String::new(),
            source_invert: false,
            source_error: None,
            source_valid: true, // Empty is valid (optional)
            destination_address: String::new(),
            destination_invert: false,
            destination_error: None,
            destination_valid: true, // Empty is valid (optional)
            rule_type: "Any Traffic".to_string(),
            service_name: String::new(),
            service_error: None,
            service_valid: true,
            port_number: String::new(),
            port_protocol: "tcp".to_string(),
            port_error: None,
            port_valid: true,
            protocol_name: "tcp".to_string(),
            protocol_error: None,
            protocol_valid: true,
            action_type: "Accept".to_string(),
            mark_value: String::new(),
            reject_type: String::new(),
            action_error: None,
            action_valid: true,
            is_valid: true, // Default rule (accept all) is valid
            tracker: 0,
        };

        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, msg: RichRuleDialogRequest, sender: AsyncComponentSender<Self>) {
        self.reset();
        match msg {
            RichRuleDialogRequest::SetFamily(family) => {
                self.set_family(family);
                self.validate_overall();
            }
            RichRuleDialogRequest::SetSourceAddress(address) => {
                self.set_source_address(address);
                sender.input(RichRuleDialogRequest::ValidateSource);
            }
            RichRuleDialogRequest::SetSourceInvert(invert) => {
                self.set_source_invert(invert);
                self.validate_overall();
            }
            RichRuleDialogRequest::ValidateSource => {
                if self.source_address.trim().is_empty() {
                    self.set_source_valid(true);
                    self.set_source_error(None);
                } else {
                    match validate_source_address(&self.source_address) {
                        Ok(_) => {
                            self.set_source_valid(true);
                            self.set_source_error(None);
                        }
                        Err(e) => {
                            self.set_source_valid(false);
                            self.set_source_error(Some(e.to_string()));
                        }
                    }
                }
                self.validate_overall();
            }
            RichRuleDialogRequest::SetDestinationAddress(address) => {
                self.set_destination_address(address);
                sender.input(RichRuleDialogRequest::ValidateDestination);
            }
            RichRuleDialogRequest::SetDestinationInvert(invert) => {
                self.set_destination_invert(invert);
                self.validate_overall();
            }
            RichRuleDialogRequest::ValidateDestination => {
                if self.destination_address.trim().is_empty() {
                    self.set_destination_valid(true);
                    self.set_destination_error(None);
                } else {
                    match validate_source_address(&self.destination_address) {
                        Ok(_) => {
                            self.set_destination_valid(true);
                            self.set_destination_error(None);
                        }
                        Err(e) => {
                            self.set_destination_valid(false);
                            self.set_destination_error(Some(e.to_string()));
                        }
                    }
                }
                self.validate_overall();
            }
            RichRuleDialogRequest::SetRuleType(rule_type) => {
                self.set_rule_type(rule_type);
                // Reset validation for all rule types
                self.set_service_valid(true);
                self.set_service_error(None);
                self.set_port_valid(true);
                self.set_port_error(None);
                self.set_protocol_valid(true);
                self.set_protocol_error(None);
                self.validate_overall();
            }
            RichRuleDialogRequest::SetService(service) => {
                self.set_service_name(service);
                sender.input(RichRuleDialogRequest::ValidateService);
            }
            RichRuleDialogRequest::ValidateService => {
                if self.rule_type == "Service" {
                    if self.service_name.trim().is_empty() {
                        self.set_service_valid(false);
                        self.set_service_error(Some("Service name is required".to_string()));
                    } else {
                        // Basic service name validation (non-empty, reasonable characters)
                        if self.service_name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
                            self.set_service_valid(true);
                            self.set_service_error(None);
                        } else {
                            self.set_service_valid(false);
                            self.set_service_error(Some("Service name can only contain letters, numbers, dashes, and underscores".to_string()));
                        }
                    }
                } else {
                    self.set_service_valid(true);
                    self.set_service_error(None);
                }
                self.validate_overall();
            }
            RichRuleDialogRequest::SetPortNumber(port) => {
                self.set_port_number(port);
                sender.input(RichRuleDialogRequest::ValidatePort);
            }
            RichRuleDialogRequest::SetPortProtocol(protocol) => {
                self.set_port_protocol(protocol);
                sender.input(RichRuleDialogRequest::ValidatePort);
            }
            RichRuleDialogRequest::ValidatePort => {
                if self.rule_type == "Port and Protocol" {
                    if self.port_number.trim().is_empty() {
                        self.set_port_valid(false);
                        self.set_port_error(Some("Port number is required".to_string()));
                    } else {
                        match validate_port(&self.port_number) {
                            Ok(_) => {
                                match validate_protocol(&self.port_protocol) {
                                    Ok(_) => {
                                        self.set_port_valid(true);
                                        self.set_port_error(None);
                                    }
                                    Err(e) => {
                                        self.set_port_valid(false);
                                        self.set_port_error(Some(e.to_string()));
                                    }
                                }
                            }
                            Err(e) => {
                                self.set_port_valid(false);
                                self.set_port_error(Some(e.to_string()));
                            }
                        }
                    }
                } else {
                    self.set_port_valid(true);
                    self.set_port_error(None);
                }
                self.validate_overall();
            }
            RichRuleDialogRequest::SetProtocol(protocol) => {
                self.set_protocol_name(protocol);
                sender.input(RichRuleDialogRequest::ValidateProtocol);
            }
            RichRuleDialogRequest::ValidateProtocol => {
                if self.rule_type == "Protocol Only" {
                    match validate_protocol(&self.protocol_name) {
                        Ok(_) => {
                            self.set_protocol_valid(true);
                            self.set_protocol_error(None);
                        }
                        Err(e) => {
                            self.set_protocol_valid(false);
                            self.set_protocol_error(Some(e.to_string()));
                        }
                    }
                } else {
                    self.set_protocol_valid(true);
                    self.set_protocol_error(None);
                }
                self.validate_overall();
            }
            RichRuleDialogRequest::SetAction(action) => {
                self.set_action_type(action);
                sender.input(RichRuleDialogRequest::ValidateAction);
            }
            RichRuleDialogRequest::SetMarkValue(value) => {
                self.set_mark_value(value);
                sender.input(RichRuleDialogRequest::ValidateAction);
            }
            RichRuleDialogRequest::SetRejectType(reject_type) => {
                self.set_reject_type(reject_type);
                self.validate_overall();
            }
            RichRuleDialogRequest::ValidateAction => {
                if self.action_type == "Mark" {
                    if self.mark_value.trim().is_empty() {
                        self.set_action_valid(false);
                        self.set_action_error(Some("Mark value is required".to_string()));
                    } else {
                        // Validate mark value (should be a number or hex)
                        if self.mark_value.parse::<u32>().is_ok() || 
                           (self.mark_value.starts_with("0x") && u32::from_str_radix(&self.mark_value[2..], 16).is_ok()) {
                            self.set_action_valid(true);
                            self.set_action_error(None);
                        } else {
                            self.set_action_valid(false);
                            self.set_action_error(Some("Mark value must be a number or hex value (e.g., 123 or 0x7b)".to_string()));
                        }
                    }
                } else {
                    self.set_action_valid(true);
                    self.set_action_error(None);
                }
                self.validate_overall();
            }
            RichRuleDialogRequest::Create => {
                if self.is_valid {
                    let rule = self.build_rich_rule();
                    let rule_xml = rule.to_xml();
                    let _ = sender.output(RichRuleDialogResponse::RichRuleCreated { rule_xml });
                    sender.input(RichRuleDialogRequest::Cancel);
                }
            }
            RichRuleDialogRequest::Cancel => {
                // Reset form
                self.set_family(String::new());
                self.set_source_address(String::new());
                self.set_source_invert(false);
                self.set_source_error(None);
                self.set_source_valid(true);
                self.set_destination_address(String::new());
                self.set_destination_invert(false);
                self.set_destination_error(None);
                self.set_destination_valid(true);
                self.set_rule_type("Any Traffic".to_string());
                self.set_service_name(String::new());
                self.set_service_error(None);
                self.set_service_valid(true);
                self.set_port_number(String::new());
                self.set_port_protocol("tcp".to_string());
                self.set_port_error(None);
                self.set_port_valid(true);
                self.set_protocol_name("tcp".to_string());
                self.set_protocol_error(None);
                self.set_protocol_valid(true);
                self.set_action_type("Accept".to_string());
                self.set_mark_value(String::new());
                self.set_reject_type(String::new());
                self.set_action_error(None);
                self.set_action_valid(true);
                self.set_is_valid(true);
            }
        }
    }
}

impl RichRuleDialog {
    fn validate_overall(&mut self) {
        // Check all component validations
        let mut valid = self.source_valid && self.destination_valid && self.action_valid;

        // Check rule type specific validations
        match self.rule_type.as_str() {
            "Service" => valid = valid && self.service_valid,
            "Port and Protocol" => valid = valid && self.port_valid,
            "Protocol Only" => valid = valid && self.protocol_valid,
            _ => {} // "Any Traffic" is always valid
        }

        // Validate logical consistency
        let source = if self.source_address.trim().is_empty() { None } else { Some(self.source_address.as_str()) };
        let destination = if self.destination_address.trim().is_empty() { None } else { Some(self.destination_address.as_str()) };
        let service = if self.rule_type == "Service" && !self.service_name.trim().is_empty() { Some(self.service_name.as_str()) } else { None };
        let port = if self.rule_type == "Port and Protocol" && !self.port_number.trim().is_empty() { 
            Some((self.port_number.as_str(), self.port_protocol.as_str())) 
        } else { 
            None 
        };
        let protocol = if self.rule_type == "Protocol Only" { Some(self.protocol_name.as_str()) } else { None };

        if let Err(_) = validate_rich_rule_logic(source, destination, service, port, protocol) {
            valid = false;
        }

        self.set_is_valid(valid);
    }

    fn build_rich_rule(&self) -> RichRule {
        let mut rule = RichRule::new();

        // Set family if specified
        if !self.family.is_empty() {
            rule = rule.with_family(self.family.clone());
        }

        // Set source if specified
        if !self.source_address.trim().is_empty() {
            rule = rule.with_source(self.source_address.trim().to_string(), self.source_invert);
        }

        // Set destination if specified
        if !self.destination_address.trim().is_empty() {
            rule = rule.with_destination(self.destination_address.trim().to_string(), self.destination_invert);
        }

        // Set traffic specification based on rule type
        match self.rule_type.as_str() {
            "Service" => {
                if !self.service_name.trim().is_empty() {
                    rule = rule.with_service(self.service_name.trim().to_string());
                }
            }
            "Port and Protocol" => {
                if !self.port_number.trim().is_empty() {
                    rule = rule.with_port(self.port_number.trim().to_string(), self.port_protocol.clone());
                }
            }
            "Protocol Only" => {
                rule = rule.with_protocol(self.protocol_name.clone());
            }
            _ => {} // "Any Traffic" - no additional specification
        }

        // Set action
        let action = match self.action_type.as_str() {
            "Accept" => RichRuleAction::Accept,
            "Reject" => {
                if self.reject_type.is_empty() {
                    RichRuleAction::Reject(None)
                } else {
                    RichRuleAction::Reject(Some(self.reject_type.clone()))
                }
            }
            "Drop" => RichRuleAction::Drop,
            "Mark" => RichRuleAction::Mark(self.mark_value.trim().to_string()),
            _ => RichRuleAction::Accept, // Default fallback
        };

        rule.with_action(action)
    }
}