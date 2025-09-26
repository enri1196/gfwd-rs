use relm4::adw::prelude::*;
use relm4::prelude::*;

use crate::core::validation::{validate_port, validate_protocol};
use crate::messages::port::{PortDialogRequest, PortDialogResponse};
use crate::models::ForwardingConfig;
use crate::utils::constants::SUPPORTED_PROTOCOLS;

#[tracker::track]
#[derive(Debug)]
pub struct AddPortDialog {
    port: String,
    protocol: String,
    is_forwarding: bool,
    dest_ip: String,
    dest_port: String,
    port_valid: bool,
    port_error: Option<String>,
    dest_ip_valid: bool,
    dest_ip_error: Option<String>,
    dest_port_valid: bool,
    dest_port_error: Option<String>,
}

#[relm4::component(async, pub)]
impl SimpleAsyncComponent for AddPortDialog {
    type Init = ();
    type Input = PortDialogRequest;
    type Output = PortDialogResponse;

    view! {
        dialog = adw::Dialog {
            set_title: "Add Port Rule",
            set_content_width: 450,
            set_content_height: 600,

            #[wrap(Some)]
            set_child = &adw::ToolbarView {
                add_top_bar = &adw::HeaderBar {
                    set_show_end_title_buttons: false,
                    add_css_class: "flat",
                    
                    #[wrap(Some)]
                    set_title_widget = &adw::WindowTitle {
                        set_title: "Add Port",
                        set_subtitle: "Configure port access and forwarding",
                    },

                    pack_start = &gtk::Button {
                        set_label: "Cancel",
                        connect_clicked[sender, root] => move |_| {
                            sender.input(PortDialogRequest::Cancel);
                            root.close();
                        },
                    },

                    pack_end = &gtk::Button {
                        set_label: "Add Port",
                        add_css_class: "suggested-action",
                        #[track(model.changed(AddPortDialog::port_valid()) | model.changed(AddPortDialog::is_forwarding()) | model.changed(AddPortDialog::dest_ip_valid()) | model.changed(AddPortDialog::dest_port_valid()))]
                        set_sensitive: model.port_valid && (!model.is_forwarding || (model.dest_ip_valid && model.dest_port_valid)),
                        connect_clicked[sender, root] => move |_| {
                            sender.input(PortDialogRequest::Add);
                            root.close();
                        },
                    },
                },

                #[wrap(Some)]
                set_content = &gtk::ScrolledWindow {
                    set_policy: (gtk::PolicyType::Never, gtk::PolicyType::Automatic),
                    set_vexpand: true,

                    #[wrap(Some)]
                    set_child = &adw::Clamp {
                        set_maximum_size: 450,
                        set_tightening_threshold: 400,

                        adw::PreferencesPage {
                            set_icon_name: Some("network-wired-symbolic"),
                            set_title: "Port Configuration",
                            set_description: "Configure port access rules and optional forwarding",

                            add = &adw::PreferencesGroup {
                                set_title: "Port Information",
                                set_description: Some("Specify which port and protocol to configure"),

                                // Port field with validation
                                add = &adw::EntryRow {
                                    set_title: "Port Number",
                                    #[watch]
                                    set_text: &model.port,
                                    #[track(model.changed(AddPortDialog::port_error()))]
                                    set_css_classes: if model.port_error.is_some() { &["error"] } else { &[] },
                                    
                                    add_prefix = &gtk::Image {
                                        set_icon_name: Some("network-server-symbolic"),
                                        set_pixel_size: 16,
                                    },

                                    add_suffix = &gtk::Label {
                                        set_text: "e.g. 80, 8080, 1000-2000",
                                        add_css_class: "dim-label",
                                        add_css_class: "caption",
                                    },

                                    connect_changed[sender] => move |entry| {
                                        sender.input(PortDialogRequest::SetPort(entry.text().to_string()));
                                    },
                                    connect_apply[sender] => move |_| {
                                        sender.input(PortDialogRequest::ValidatePort);
                                    },
                                },

                                // Port validation feedback
                                add = &adw::ActionRow {
                                    #[track(model.changed(AddPortDialog::port_error()))]
                                    set_visible: model.port_error.is_some(),
                                    #[track(model.changed(AddPortDialog::port_error()))]
                                    set_title: model.port_error.as_deref().unwrap_or(""),
                                    add_css_class: "error",

                                    add_prefix = &gtk::Image {
                                        set_icon_name: Some("dialog-warning-symbolic"),
                                        set_pixel_size: 16,
                                        add_css_class: "error",
                                    },
                                },

                                // Protocol selection
                                add = &adw::ComboRow {
                                    set_title: "Protocol",
                                    set_subtitle: "Network protocol for this port",
                                    set_model: Some(&gtk::StringList::new(SUPPORTED_PROTOCOLS)),
                                    set_selected: SUPPORTED_PROTOCOLS.iter().position(|&p| p == model.protocol.as_str()).unwrap_or(0) as u32,
                                    
                                    add_prefix = &gtk::Image {
                                        set_icon_name: Some("preferences-system-network-symbolic"),
                                        set_pixel_size: 16,
                                    },

                                    connect_selected_notify[sender] => move |combo| {
                                        let protocol = SUPPORTED_PROTOCOLS.get(combo.selected() as usize).unwrap_or(&"tcp");
                                        sender.input(PortDialogRequest::SetProtocol(protocol.to_string()));
                                    },
                                },
                            },

                            add = &adw::PreferencesGroup {
                                set_title: "Port Forwarding",
                                set_description: Some("Optionally forward traffic to another destination"),

                                // Port forwarding toggle
                                add = &adw::SwitchRow {
                                    set_title: "Enable Port Forwarding",
                                    set_subtitle: "Forward incoming traffic to another address",
                                    #[track(model.changed(AddPortDialog::is_forwarding()))]
                                    set_active: model.is_forwarding,
                                    
                                    add_prefix = &gtk::Image {
                                        set_icon_name: Some("go-jump-symbolic"),
                                        set_pixel_size: 16,
                                    },

                                    connect_active_notify[sender] => move |switch| {
                                        sender.input(PortDialogRequest::SetIsForwarding(switch.is_active()));
                                    },
                                },
                            },

                            // Forwarding configuration (conditionally visible)
                            add = &adw::PreferencesGroup {
                                set_title: "Forwarding Destination",
                                set_description: Some("Configure where to forward the traffic"),
                                #[track(model.changed(AddPortDialog::is_forwarding()))]
                                set_visible: model.is_forwarding,

                                // Destination IP
                                add = &adw::EntryRow {
                                    set_title: "Destination IP Address",
                                    #[watch]
                                    set_text: &model.dest_ip,
                                    #[track(model.changed(AddPortDialog::dest_ip_error()))]
                                    set_css_classes: if model.dest_ip_error.is_some() { &["error"] } else { &[] },
                                    
                                    add_prefix = &gtk::Image {
                                        set_icon_name: Some("network-workgroup-symbolic"),
                                        set_pixel_size: 16,
                                    },

                                    add_suffix = &gtk::Label {
                                        set_text: "e.g. 192.168.1.100",
                                        add_css_class: "dim-label",
                                        add_css_class: "caption",
                                    },

                                    connect_changed[sender] => move |entry| {
                                        sender.input(PortDialogRequest::SetDestIp(entry.text().to_string()));
                                    },
                                    connect_apply[sender] => move |_| {
                                        sender.input(PortDialogRequest::ValidateDestIp);
                                    },
                                },

                                // Destination IP validation feedback
                                add = &adw::ActionRow {
                                    #[track(model.changed(AddPortDialog::dest_ip_error()))]
                                    set_visible: model.dest_ip_error.is_some(),
                                    #[track(model.changed(AddPortDialog::dest_ip_error()))]
                                    set_title: model.dest_ip_error.as_deref().unwrap_or(""),
                                    add_css_class: "error",

                                    add_prefix = &gtk::Image {
                                        set_icon_name: Some("dialog-warning-symbolic"),
                                        set_pixel_size: 16,
                                        add_css_class: "error",
                                    },
                                },

                                // Destination port
                                add = &adw::EntryRow {
                                    set_title: "Destination Port",
                                    #[watch]
                                    set_text: &model.dest_port,
                                    #[track(model.changed(AddPortDialog::dest_port_error()))]
                                    set_css_classes: if model.dest_port_error.is_some() { &["error"] } else { &[] },
                                    
                                    add_prefix = &gtk::Image {
                                        set_icon_name: Some("network-server-symbolic"),
                                        set_pixel_size: 16,
                                    },

                                    add_suffix = &gtk::Label {
                                        set_text: "e.g. 8080",
                                        add_css_class: "dim-label",
                                        add_css_class: "caption",
                                    },

                                    connect_changed[sender] => move |entry| {
                                        sender.input(PortDialogRequest::SetDestPort(entry.text().to_string()));
                                    },
                                    connect_apply[sender] => move |_| {
                                        sender.input(PortDialogRequest::ValidateDestPort);
                                    },
                                },

                                // Destination port validation feedback
                                add = &adw::ActionRow {
                                    #[track(model.changed(AddPortDialog::dest_port_error()))]
                                    set_visible: model.dest_port_error.is_some(),
                                    #[track(model.changed(AddPortDialog::dest_port_error()))]
                                    set_title: model.dest_port_error.as_deref().unwrap_or(""),
                                    add_css_class: "error",

                                    add_prefix = &gtk::Image {
                                        set_icon_name: Some("dialog-warning-symbolic"),
                                        set_pixel_size: 16,
                                        add_css_class: "error",
                                    },
                                },

                                // Forwarding summary
                                add = &adw::ActionRow {
                                    #[track(model.changed(AddPortDialog::is_forwarding()) | model.changed(AddPortDialog::dest_ip()) | model.changed(AddPortDialog::dest_port()) | model.changed(AddPortDialog::port()) | model.changed(AddPortDialog::protocol()))]
                                    set_visible: model.is_forwarding && !model.dest_ip.is_empty() && !model.dest_port.is_empty(),
                                    set_title: "Forwarding Summary",
                                    #[track(model.changed(AddPortDialog::dest_ip()) | model.changed(AddPortDialog::dest_port()) | model.changed(AddPortDialog::port()) | model.changed(AddPortDialog::protocol()))]
                                    set_subtitle: &format!("{}:{}/{} → {}:{}", 
                                        "0.0.0.0", // Any interface
                                        model.port,
                                        model.protocol,
                                        model.dest_ip,
                                        model.dest_port
                                    ),
                                    add_css_class: "caption",

                                    add_prefix = &gtk::Image {
                                        set_icon_name: Some("emblem-ok-symbolic"),
                                        set_pixel_size: 16,
                                        add_css_class: "success",
                                    },
                                },
                            },
                        },
                    },
                },
            },
        }
    }

    async fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let model = AddPortDialog {
            port: String::new(),
            protocol: "tcp".to_string(),
            is_forwarding: false,
            dest_ip: String::new(),
            dest_port: String::new(),
            port_valid: false,
            port_error: None,
            dest_ip_valid: false,
            dest_ip_error: None,
            dest_port_valid: false,
            dest_port_error: None,
            tracker: 0,
        };

        let widgets = view_output!();
        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, msg: Self::Input, sender: AsyncComponentSender<Self>) {
        self.reset();

        match msg {
            PortDialogRequest::SetPort(port) => {
                self.set_port(port);
                sender.input(PortDialogRequest::ValidatePort);
            }
            PortDialogRequest::SetProtocol(protocol) => {
                // Validate protocol for extra safety
                match validate_protocol(&protocol) {
                    Ok(validated_protocol) => self.set_protocol(validated_protocol),
                    Err(_) => {
                        // Fallback to tcp if invalid protocol somehow gets through
                        self.set_protocol("tcp".to_string());
                    }
                }
            }
            PortDialogRequest::SetIsForwarding(is_forwarding) => {
                self.set_is_forwarding(is_forwarding);
                if is_forwarding {
                    sender.input(PortDialogRequest::ValidateDestIp);
                    sender.input(PortDialogRequest::ValidateDestPort);
                }
            }
            PortDialogRequest::SetDestIp(dest_ip) => {
                self.set_dest_ip(dest_ip);
                sender.input(PortDialogRequest::ValidateDestIp);
            }
            PortDialogRequest::SetDestPort(dest_port) => {
                self.set_dest_port(dest_port);
                sender.input(PortDialogRequest::ValidateDestPort);
            }
            PortDialogRequest::ValidatePort => match validate_port(&self.port) {
                Ok(_) => {
                    self.set_port_valid(true);
                    self.set_port_error(None);
                }
                Err(e) => {
                    self.set_port_valid(false);
                    self.set_port_error(Some(e.user_message()));
                }
            },
            PortDialogRequest::ValidateDestIp => {
                if self.dest_ip.trim().is_empty() {
                    self.set_dest_ip_valid(false);
                    self.set_dest_ip_error(Some(
                        "Destination IP is required for forwarding".to_string(),
                    ));
                } else {
                    // Basic IP validation - you could use a more sophisticated validator
                    if self.dest_ip.parse::<std::net::IpAddr>().is_ok() {
                        self.set_dest_ip_valid(true);
                        self.set_dest_ip_error(None);
                    } else {
                        self.set_dest_ip_valid(false);
                        self.set_dest_ip_error(Some("Invalid IP address format".to_string()));
                    }
                }
            }
            PortDialogRequest::ValidateDestPort => match validate_port(&self.dest_port) {
                Ok(_) => {
                    self.set_dest_port_valid(true);
                    self.set_dest_port_error(None);
                }
                Err(e) => {
                    self.set_dest_port_valid(false);
                    self.set_dest_port_error(Some(e.user_message()));
                }
            },
            PortDialogRequest::Add => {
                if self.port_valid
                    && (!self.is_forwarding || (self.dest_ip_valid && self.dest_port_valid))
                {
                    let forwarding = if self.is_forwarding {
                        Some(ForwardingConfig {
                            to_addr: self.dest_ip.clone(),
                            to_port: self.dest_port.clone(),
                        })
                    } else {
                        None
                    };

                    sender
                        .output(PortDialogResponse::PortAdded {
                            port: self.port.clone(),
                            protocol: self.protocol.clone(),
                            forwarding,
                        })
                        .unwrap();
                }
            }
            PortDialogRequest::Cancel => {
                // Reset form
                self.set_port(String::new());
                self.set_protocol("tcp".to_string());
                self.set_is_forwarding(false);
                self.set_dest_ip(String::new());
                self.set_dest_port(String::new());
                self.set_port_valid(false);
                self.set_port_error(None);
                self.set_dest_ip_valid(false);
                self.set_dest_ip_error(None);
                self.set_dest_port_valid(false);
                self.set_dest_port_error(None);
            }
        }
    }
}