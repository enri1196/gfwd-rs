use relm4::adw::prelude::*;
use relm4::gtk::gio::MenuModel;
use relm4::prelude::*;

use crate::components::zone_view::port_item::PortItem;
use crate::fwd_broker::ZoneSettings;

#[tracker::track]
#[derive(Debug)]
pub struct ZoneInfoComponent {
    pub(crate) settings: Option<ZoneSettings>,
    #[tracker::do_not_track]
    pub(crate) ports: FactoryVecDeque<PortItem>,
}

#[derive(Clone, Debug)]
pub enum ZoneInfoRequest {
    SetSettings(ZoneSettings),
    AddPort {
        port: String,
        protocol: String,
        forward_port: Option<ForwardOpts>,
    },
    RemovePort {
        port: String,
        protocol: String,
        forward_port: Option<ForwardOpts>,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct ForwardOpts {
    pub to_port: String,
    pub to_addr: String,
}

#[derive(Debug)]
pub enum ZoneInfoResponse {
    AddPort {
        port: String,
        protocol: String,
        forward_port: Option<ForwardOpts>,
    },
    RemovePort {
        port: String,
        protocol: String,
        forward_port: Option<ForwardOpts>,
    },
}

#[relm4::component(pub)]
impl SimpleComponent for ZoneInfoComponent {
    type Init = ();
    type Input = ZoneInfoRequest;
    type Output = ZoneInfoResponse;

    view! {
        adw::PreferencesPage {
            add = &adw::PreferencesGroup {
                set_title: "General",
                set_description: Some("Basic zone information"),

                #[name = "name_row"]
                adw::ActionRow {
                    set_title: "Name",
                    #[watch]
                    set_subtitle: &model.settings.as_ref().map_or_else(|| "".to_string(), |s| s.name.clone()),
                },

                #[name = "description_row"]
                adw::ActionRow {
                    set_title: "Description",
                    #[watch]
                    set_subtitle: &model.settings.as_ref().map_or_else(|| "".to_string(), |s| s.description.clone()),
                },
            },

            add = &adw::PreferencesGroup {
                set_title: "Behavior",

                #[name = "target_row"]
                adw::ActionRow {
                    set_title: "Target",
                    #[watch]
                    set_subtitle: &model.settings.as_ref().map_or_else(|| "".to_string(), |s| s.target.to_string()),
                },

                #[name = "masquerade_row"]
                adw::ActionRow {
                    set_title: "Masquerade",
                    #[watch]
                    set_subtitle: &model.settings.as_ref().map_or_else(|| "".to_string(), |s| if s.masquerade { "Enabled" } else { "Disabled" }.to_string()),
                },
            },

            add = &adw::PreferencesGroup {
                set_title: "Allowed Ports",

                #[name = "add_menu_btn"]
                #[wrap(Some)]
                set_header_suffix = &gtk::MenuButton {
                    set_icon_name: "list-add-symbolic",
                    set_tooltip_text: Some("Add port"),

                    #[wrap(Some)]
                    set_popover = &gtk::PopoverMenu::from_model(None::<&MenuModel>) {
                        set_position: gtk::PositionType::Bottom,
                        #[wrap(Some)]
                        set_child = &gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            set_spacing: 8,
                            set_margin_all: 8,

                            gtk::Box {
                                set_spacing: 6,
                                #[name = "protocol_toggle"]
                                gtk::ToggleButton {
                                    set_label: "TCP",
                                    set_active: true,
                                    connect_toggled => move |btn| {
                                        if btn.is_active() {
                                            btn.set_label("TCP");
                                        } else {
                                            btn.set_label("UDP");
                                        }
                                    }
                                },
                                #[name = "port_entry"]
                                gtk::Entry { set_placeholder_text: Some("80 or 8000-8080") }
                            },

                            #[name = "forwarding_section"]
                            gtk::Box {
                                set_orientation: gtk::Orientation::Vertical,
                                set_spacing: 6,

                                #[name = "forwarding_toggle"]
                                gtk::ToggleButton {
                                    set_label: "Port Forwarding: Disabled",
                                    set_active: false,
                                    connect_toggled[dest_ip_entry, dest_port_entry, port_entry] => move |btn| {
                                        let is_enabled = btn.is_active();
                                        if is_enabled {
                                            btn.set_label("Port Forwarding: Enabled");
                                            // Auto-populate destination port with source port
                                            let source_port = port_entry.text().to_string();
                                            if !source_port.is_empty() {
                                                dest_port_entry.set_text(&source_port);
                                            }
                                        } else {
                                            btn.set_label("Port Forwarding: Disabled");
                                            // Clear forwarding fields when disabled
                                            dest_ip_entry.set_text("");
                                            dest_port_entry.set_text("");
                                        }
                                        dest_ip_entry.set_visible(is_enabled);
                                        dest_port_entry.set_visible(is_enabled);
                                    }
                                },

                                #[name = "dest_ip_entry"]
                                gtk::Entry {
                                    set_placeholder_text: Some("Destination IP (e.g. 192.168.1.100)"),
                                    set_visible: false,
                                },

                                #[name = "dest_port_entry"]
                                gtk::Entry {
                                    set_placeholder_text: Some("Destination port (e.g. 8080)"),
                                    set_visible: false,
                                }
                            },

                            gtk::Box {
                                set_orientation: gtk::Orientation::Horizontal,
                                set_halign: gtk::Align::Center,
                                set_spacing: 6,
                                gtk::Button { set_label: "Cancel", connect_clicked[add_menu_btn] => move |_| { add_menu_btn.popdown(); } },
                                gtk::Button {
                                    add_css_class: "suggested-action",
                                    set_label: "Add",
                                    connect_clicked[sender, protocol_toggle, port_entry, forwarding_toggle, dest_ip_entry, dest_port_entry, add_menu_btn] => move |_| {
                                        let protocol = if protocol_toggle.is_active() { "tcp" } else { "udp" }.to_string();
                                        let port = port_entry.text().to_string();

                                        if port.trim().is_empty() {
                                            return;
                                        }

                                        let forward_port = if forwarding_toggle.is_active() {
                                            let dest_ip = dest_ip_entry.text().to_string();
                                            let dest_port = dest_port_entry.text().to_string();

                                            if dest_ip.trim().is_empty() || dest_port.trim().is_empty() {
                                                return;
                                            }

                                            Some(ForwardOpts {
                                                to_addr: dest_ip,
                                                to_port: dest_port
                                            })
                                        } else {
                                            None
                                        };

                                        sender.input(ZoneInfoRequest::AddPort {
                                            port,
                                            protocol,
                                            forward_port
                                        });
                                        add_menu_btn.popdown();
                                    }
                                }
                            }
                        }
                    }
                },

                #[local_ref]
                ports_list_box -> gtk::ListBox {
                    set_selection_mode: gtk::SelectionMode::None,
                    add_css_class: "boxed-list",
                }
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let ports = FactoryVecDeque::builder().launch_default().forward(
            sender.input_sender(),
            |(port, protocol, forward_port): (String, String, Option<ForwardOpts>)| {
                ZoneInfoRequest::RemovePort {
                    port,
                    protocol,
                    forward_port,
                }
            },
        );

        let model = ZoneInfoComponent {
            settings: None,
            ports,
            tracker: 0,
        };

        let ports_list_box = model.ports.widget();
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        self.reset();
        match message {
            ZoneInfoRequest::SetSettings(settings) => {
                let mut ports = self.ports.guard();
                ports.clear();

                // Load regular ports
                for (port, protocol) in &settings.ports {
                    ports.push_back((port.clone(), protocol.clone(), None));
                }

                // Load forwarded ports
                for (port, protocol, to_port, to_addr) in &settings.forward_ports {
                    ports.push_back((
                        port.clone(),
                        protocol.clone(),
                        Some(ForwardOpts {
                            to_port: to_port.clone(),
                            to_addr: to_addr.clone(),
                        }),
                    ));
                }

                drop(ports);
                self.set_settings(Some(settings.clone()));
            }
            ZoneInfoRequest::AddPort {
                port,
                protocol,
                forward_port,
            } => {
                let _ = _sender.output(ZoneInfoResponse::AddPort {
                    port: port.clone(),
                    protocol: protocol.clone(),
                    forward_port: forward_port.clone(),
                });
                self.ports.guard().push_back((port, protocol, forward_port));
            }
            ZoneInfoRequest::RemovePort {
                port,
                protocol,
                forward_port,
            } => {
                let _ = _sender.output(ZoneInfoResponse::RemovePort {
                    port: port.clone(),
                    protocol: protocol.clone(),
                    forward_port: forward_port.clone(),
                });
                let mut ports = self.ports.guard();
                let Some(index) = ports
                    .iter()
                    .position(|item| item.port == port && item.protocol == protocol)
                else {
                    return;
                };
                ports.remove(index);
            }
        }
    }
}
