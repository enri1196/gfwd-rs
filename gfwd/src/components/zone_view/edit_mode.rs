use relm4::adw::prelude::*;
use relm4::prelude::*;

use crate::fwd_broker::{ZoneSettings, ZoneTarget};

#[tracker::track]
#[derive(Debug)]
pub struct ZoneEditMode {
    pub(crate) settings: Option<ZoneSettings>,
}

#[derive(Debug)]
pub enum ZoneEditModeMsg {
    SetSettings(ZoneSettings),
}

#[relm4::component(pub)]
impl SimpleComponent for ZoneEditMode {
    type Init = ();
    type Input = ZoneEditModeMsg;
    type Output = ();

    view! {
        adw::PreferencesPage {
            add = &adw::PreferencesGroup {
                set_title: "General",

                adw::EntryRow {
                    set_title: "Name",
                    #[watch]
                    set_text: &model.settings.as_ref().map_or_else(|| "".to_string(), |s| s.name.clone()),
                    // connect_changed to update model
                },

                adw::EntryRow {
                    set_title: "Description",
                    #[watch]
                    set_text: &model.settings.as_ref().map_or_else(|| "".to_string(), |s| s.description.clone()),
                    // connect_changed to update model
                },
            },
            add = &adw::PreferencesGroup {
                set_title: "Behavior",
                adw::ComboRow {
                    set_title: "Target",
                    set_model: Some(&gtk::StringList::new(&["default", "ACCEPT", "DROP", "REJECT"])),
                    #[watch]
                    set_selected: model.settings.as_ref().map_or(0, |s| {
                        match s.target {
                            ZoneTarget::Default => 0,
                            ZoneTarget::Accept => 1,
                            ZoneTarget::Drop => 2,
                            ZoneTarget::Reject => 3,
                        }
                    }),
                },
                adw::SwitchRow {
                    set_title: "Masquerade",
                    #[watch]
                    set_active: model.settings.as_ref().map_or(false, |s| s.masquerade),
                }
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = ZoneEditMode {
            settings: None,
            tracker: 0,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        self.reset();
        match message {
            ZoneEditModeMsg::SetSettings(settings) => self.set_settings(Some(settings)),
        }
    }
}
