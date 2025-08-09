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
}

#[relm4::component(pub)]
impl SimpleComponent for ZoneViewMode {
    type Init = ();
    type Input = ZoneViewModeMsg;
    type Output = ();

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
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let ports =
            FactoryVecDeque::builder()
                .launch_default()
                .forward(_sender.input_sender(), |_| {
                    // No output from PortItem needed for now
                    todo!()
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
        }
    }
}
