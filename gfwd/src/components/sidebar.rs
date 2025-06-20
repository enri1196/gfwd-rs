use std::collections::HashMap;

use crate::components::zone_item::ZoneItem;
use crate::fwd_broker::FwdBroker;
use relm4::adw::prelude::*;
use relm4::gtk::glib::{self, LogLevel};
use relm4::prelude::*;

pub struct SidebarView {
    broker: &'static FwdBroker,
    active_zones: HashMap<String, HashMap<String, Vec<String>>>,
    zones: FactoryVecDeque<ZoneItem>,
    default_zone: String,
}

#[derive(Debug)]
pub enum SidebarViewRequest {
    UpdateZones,
    ShowAddZoneDialog,
    SetDefaultZone,
    SetActiveZones,
}

#[derive(Debug)]
pub enum SidebarViewResponse {
    ShowAddZoneDialog,
    SelectedZone(String),
}

#[relm4::component(async, pub)]
impl SimpleAsyncComponent for SidebarView {
    type Init = ();
    type Input = SidebarViewRequest;
    type Output = SidebarViewResponse;
    type Widgets = SidebarWidgets;

    view! {
        gtk::ScrolledWindow {
            set_vexpand: true,
            set_hscrollbar_policy: gtk::PolicyType::Never,
            #[wrap(Some)]
            set_child = &gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 12,
                set_margin_all: 12,
                set_width_request: 250,

                adw::HeaderBar {
                    set_show_end_title_buttons: false,
                    set_css_classes: &["flat"],

                    #[wrap(Some)]
                    set_title_widget = &gtk::Label {
                        set_text: "Firewall Zones",
                        set_css_classes: &["title-2"],
                        set_halign: gtk::Align::Start,
                    },

                    pack_end = &gtk::Button {
                        set_icon_name: "list-add-symbolic",
                        set_tooltip_text: Some("New Zone"),
                        set_css_classes: &["flat"],
                        connect_clicked[sender] => move |_| {
                            sender.input(SidebarViewRequest::ShowAddZoneDialog);
                        }
                    }
                },

                #[local_ref]
                zones_list_box -> gtk::ListBox {}
            }
        }
    }

    async fn update(&mut self, msg: Self::Input, sender: AsyncComponentSender<Self>) {
        match msg {
            SidebarViewRequest::UpdateZones => {
                let zones = self.broker.get_zones().await.unwrap_or_default();
                glib::g_log!(LogLevel::Message, "Found {} zones", zones.len());
                for zone in zones {
                    if !self.zones.iter().any(|item| item.get_name() == &zone) {
                        self.zones.guard().push_back(zone);
                    }
                }
                sender.input(SidebarViewRequest::SetDefaultZone);
            }
            SidebarViewRequest::SetDefaultZone => {
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
                sender.input(SidebarViewRequest::SetActiveZones);
            }
            SidebarViewRequest::SetActiveZones => {
                match self.broker.get_active_zones().await {
                    Ok(active_zones) => {
                        glib::g_log!(LogLevel::Message, "Active zones: {}", active_zones.len());
                        glib::g_log!(LogLevel::Message, "{:?}", active_zones);
                        self.active_zones = active_zones;
                        if let Some(_zone) = self.active_zones.get(&self.default_zone) {
                            // glib::g_log!(LogLevel::Message, "Public zone: {}", zone.len());
                            self.zones.guard().iter_mut().for_each(|zone| {
                                zone.set_is_active(zone.name == self.default_zone)
                            });
                        }
                    }
                    Err(error) => glib::g_log!(LogLevel::Error, "Active Zone Error: {}", error),
                }
            }
            SidebarViewRequest::ShowAddZoneDialog => {
                let _ = sender.output(SidebarViewResponse::ShowAddZoneDialog);
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
                    crate::components::zone_item::ZoneItemResponse::SelectedZone(item_name) => {
                        SidebarViewResponse::SelectedZone(item_name)
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
        sender.input(SidebarViewRequest::UpdateZones);
        AsyncComponentParts { model, widgets }
    }
}
