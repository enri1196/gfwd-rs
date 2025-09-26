use relm4::adw::prelude::*;

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
    ShowAddPortDialog,
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
    ShowAddPortDialog,
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

                #[wrap(Some)]
                set_header_suffix = &gtk::Button {
                    set_icon_name: "list-add-symbolic",
                    set_tooltip_text: Some("Add port"),
                    add_css_class: "flat",
                    connect_clicked[sender] => move |_| {
                        sender.input(ZoneInfoRequest::ShowAddPortDialog);
                    },
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
            ZoneInfoRequest::ShowAddPortDialog => {
                let _ = _sender.output(ZoneInfoResponse::ShowAddPortDialog);
            }
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
