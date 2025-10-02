use std::collections::HashMap;

use relm4::adw::prelude::*;
use relm4::gtk::glib::{self, LogLevel};
use relm4::prelude::*;

use crate::core::FwdBroker;
use crate::messages::zone::{SidebarRequest, SidebarResponse};
use crate::ui::components::ZoneItem;
use crate::utils::constants::SIDEBAR_WIDTH;

pub struct SidebarView {
    broker: &'static FwdBroker,
    active_zones: HashMap<String, HashMap<String, Vec<String>>>,
    zones: FactoryVecDeque<ZoneItem>,
    default_zone: String,
}

#[relm4::component(async, pub)]
impl SimpleAsyncComponent for SidebarView {
    type Init = ();
    type Input = SidebarRequest;
    type Output = SidebarResponse;

    view! {
        adw::ToolbarView {
            add_top_bar = &adw::HeaderBar {
                set_show_end_title_buttons: false,
                add_css_class: "flat",

                #[wrap(Some)]
                set_title_widget = &adw::WindowTitle {
                    set_title: "Firewall Zones",
                },

                pack_start = &gtk::Button {
                    set_icon_name: "network-server-symbolic",
                    set_tooltip_text: Some("IP Sets"),
                    add_css_class: "flat",
                    connect_clicked[sender] => move |_| {
                        sender.input(SidebarRequest::ShowIPSets);
                    }
                },

                pack_end = &gtk::Button {
                    set_icon_name: "list-add-symbolic",
                    set_tooltip_text: Some("Add New Zone"),
                    add_css_class: "flat",
                    connect_clicked[sender] => move |_| {
                        sender.input(SidebarRequest::ShowAddZoneDialog);
                    }
                }
            },

            #[wrap(Some)]
            set_content = &gtk::ScrolledWindow {
                set_policy: (gtk::PolicyType::Never, gtk::PolicyType::Automatic),
                set_vexpand: true,
                set_width_request: SIDEBAR_WIDTH,

                #[wrap(Some)]
                set_child = &adw::Clamp {
                    set_maximum_size: SIDEBAR_WIDTH,
                    set_tightening_threshold: SIDEBAR_WIDTH - 50,

                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 0,
                        set_margin_all: 12,

                        // Main zones list
                        #[local_ref]
                        zones_list_box -> gtk::ListBox {
                            set_selection_mode: gtk::SelectionMode::None,
                            add_css_class: "boxed-list",
                            set_margin_bottom: 18,
                        },

                        // Status legend
                        adw::PreferencesGroup {
                            set_title: "Legend",
                            set_margin_top: 6,

                            adw::ActionRow {
                                set_title: "Default Zone",
                                set_subtitle: "Primary firewall zone",
                                add_prefix = &gtk::Image {
                                    set_icon_name: Some("security-high-symbolic"),
                                    set_pixel_size: 16,
                                    add_css_class: "accent",
                                },
                            },

                            adw::ActionRow {
                                set_title: "Active Zone",
                                set_subtitle: "Currently in use",
                                add_prefix = &gtk::Image {
                                    set_icon_name: Some("object-select-symbolic"),
                                    set_pixel_size: 16,
                                    add_css_class: "success",
                                },
                            },
                        },
                    },
                },
            },
        }
    }

    async fn update(&mut self, msg: Self::Input, sender: AsyncComponentSender<Self>) {
        match msg {
            SidebarRequest::UpdateZones => {
                let zones = self.broker.get_zones().await.unwrap_or_default();
                glib::g_log!(LogLevel::Message, "Found {} zones", zones.len());
                for zone in zones {
                    if !self.zones.iter().any(|item| item.get_name() == &zone) {
                        self.zones.guard().push_back(zone);
                    }
                }
                sender.input(SidebarRequest::SetDefaultZone);
            }
            SidebarRequest::SetDefaultZone => {
                match self.broker.get_default_zone().await {
                    Ok(default_zone) => {
                        glib::g_log!(LogLevel::Message, "Default zone: {}", default_zone);
                        for zone in self.zones.guard().iter_mut() {
                            zone.set_is_default(zone.name == default_zone);
                        }
                        self.default_zone = default_zone;
                    }
                    Err(error) => glib::g_log!(LogLevel::Error, "Default Zone Error: {}", error),
                }
                sender.input(SidebarRequest::SetActiveZones);
            }
            SidebarRequest::SetActiveZones => match self.broker.get_active_zones().await {
                Ok(active_zones) => {
                    glib::g_log!(LogLevel::Message, "Active zones: {}", active_zones.len());
                    glib::g_log!(LogLevel::Message, "{:?}", active_zones);
                    self.active_zones = active_zones;
                    if let Some(_zone) = self.active_zones.get(&self.default_zone) {
                        self.zones
                            .guard()
                            .iter_mut()
                            .for_each(|zone| zone.set_is_active(zone.name == self.default_zone));
                    }
                }
                Err(error) => glib::g_log!(LogLevel::Error, "Active Zone Error: {}", error),
            },
            SidebarRequest::ShowAddZoneDialog => {
                let _ = sender.output(SidebarResponse::ShowAddZoneDialog);
            }
            SidebarRequest::ShowIPSets => {
                let _ = sender.output(SidebarResponse::ShowIPSets);
            }
            SidebarRequest::RemoveZone(removed_zone) => {
                let mut zones = self.zones.guard();
                let Some(idx) = zones.iter().position(|item| &item.name == &removed_zone) else {
                    glib::g_log!(
                        LogLevel::Error,
                        "Sidebar could not remove zone named: {}",
                        removed_zone
                    );
                    return;
                };
                zones.remove(idx);
            }
        }
    }

    async fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let broker = FwdBroker::get_broker().await;

        let zones =
            FactoryVecDeque::builder()
                .launch_default()
                .forward(sender.output_sender(), |msg| match msg {
                    crate::ui::components::ZoneItemResponse::SelectedZone(item_name) => {
                        SidebarResponse::SelectedZone(item_name)
                    }
                });

        let model = SidebarView {
            broker,
            active_zones: HashMap::new(),
            zones,
            default_zone: String::new(),
        };

        let zones_list_box = model.zones.widget();
        let widgets = view_output!();
        sender.input(SidebarRequest::UpdateZones);
        AsyncComponentParts { model, widgets }
    }
}
