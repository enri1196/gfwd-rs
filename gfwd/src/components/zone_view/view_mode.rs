use relm4::adw::prelude::*;
use relm4::prelude::*;

use crate::components::zone_view::port_item::PortItem;
use crate::fwd_broker::ZoneSettings;

#[tracker::track]
#[derive(Debug)]
pub struct ZoneViewMode {
    pub(crate) settings: Option<ZoneSettings>,
    #[tracker::do_not_track]
    pub(crate) ports: FactoryVecDeque<PortItem>,
}

#[derive(Debug)]
pub enum ZoneViewModeMsg {
    SetSettings(ZoneSettings),
    #[allow(unused)]
    AddPortClicked,
    AddPortConfirmed(String, String),
    RemovePort(String, String),
}

#[derive(Debug)]
pub enum ZoneViewModeOut {
    AddPort(String, String),
    RemovePort(String, String),
}

#[relm4::component(pub)]
impl SimpleComponent for ZoneViewMode {
    type Init = ();
    type Input = ZoneViewModeMsg;
    type Output = ZoneViewModeOut;

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
                    set_popover = &gtk::Popover {
                        #[wrap(Some)]
                        set_child = &gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            set_spacing: 8,
                            set_margin_all: 8,

                            gtk::Box {
                                set_spacing: 6,
                                gtk::Label { set_label: "Protocol:" },
                                #[name = "protocol_dd"]
                                gtk::DropDown {
                                    set_model: Some(&gtk::StringList::new(&["tcp", "udp"])),
                                    set_selected: 0,
                                }
                            },

                            gtk::Box {
                                set_spacing: 6,
                                gtk::Label { set_label: "Port/range:" },
                                #[name = "port_entry"]
                                gtk::Entry { set_placeholder_text: Some("80 or 8000-8080") }
                            },

                            gtk::Box {
                                set_orientation: gtk::Orientation::Horizontal,
                                set_halign: gtk::Align::End,
                                set_spacing: 6,
                                gtk::Button { set_label: "Cancel", connect_clicked[add_menu_btn] => move |_| { add_menu_btn.popdown(); } },
                                gtk::Button {
                                    add_css_class: "suggested-action",
                                    set_label: "Add",
                                    connect_clicked[sender, protocol_dd, port_entry, add_menu_btn] => move |_| {
                                        let idx = protocol_dd.selected();
                                        let protocol = if idx == 1 { "udp" } else { "tcp" }.to_string();
                                        let port = port_entry.text().to_string();
                                        sender.input(ZoneViewModeMsg::AddPortConfirmed(port, protocol));
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
        let ports = FactoryVecDeque::builder()
            .launch_default()
            .forward(sender.input_sender(), |output: (String, String)| {
                ZoneViewModeMsg::RemovePort(output.0, output.1)
            });

        let model = ZoneViewMode {
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
            ZoneViewModeMsg::SetSettings(settings) => {
                let mut ports = self.ports.guard();
                ports.clear();
                for (port, protocol) in &settings.ports {
                    ports.push_back((port.clone(), protocol.clone()));
                }
                drop(ports);

                self.set_settings(Some(settings.clone()));
            }
            ZoneViewModeMsg::AddPortClicked => {}
            ZoneViewModeMsg::AddPortConfirmed(port, protocol) => {
                let _ = _sender.output(ZoneViewModeOut::AddPort(port.clone(), protocol.clone()));
                self.ports.guard().push_back((port, protocol));
            }
            ZoneViewModeMsg::RemovePort(port, protocol) => {
                let _ = _sender.output(ZoneViewModeOut::RemovePort(port.clone(), protocol.clone()));
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
