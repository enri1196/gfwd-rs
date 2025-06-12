use relm4::adw::prelude::*;
use relm4::prelude::*;
use crate::fwd_broker::FwdBroker;

#[tracker::track]
#[derive(Debug, Clone, PartialEq)]
pub struct ZoneItem {
    pub name: String,
    pub is_default: bool,
    pub is_active: bool,
    pub interfaces: Vec<String>,
}

impl From<String> for ZoneItem {
    fn from(value: String) -> Self {
        Self {
            name: value,
            is_default: false,
            is_active: false,
            interfaces: Vec::new(),
            tracker: 0,
        }
    }
}

#[relm4::factory(pub)]
impl FactoryComponent for ZoneItem {
    type Init = String;
    type Input = ();
    type Output = ();
    type CommandOutput = ();
    type ParentWidget = gtk::ListBox;

    view! {
        gtk::ListBoxRow {
            set_halign: gtk::Align::Fill,
            set_margin_all: 4,

            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 12,
                set_margin_all: 8,
                set_hexpand: true,
                
                #[name(label)]
                gtk::Label {
                    set_label: &self.name,
                    set_hexpand: true,
                    set_halign: gtk::Align::Start,
                },

                #[name(default_icon)]
                gtk::Image {
                    set_icon_name: Some("object-select-symbolic"),
                    #[track(self.changed(ZoneItem::is_default()))]
                    set_visible: self.is_default,
                    #[track(self.changed(ZoneItem::is_default()))]
                    set_opacity: if self.is_default { 1.0 } else { 0.0 },
                },
            }
        }
    }

    fn init_model(name: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        Self {
            name,
            is_default: false,
            is_active: false,
            interfaces: Vec::new(),
            tracker: 0,
        }
    }
}

#[derive(Debug)]
pub enum InputSidebarMsg {
    UpdateZones,
    ShowAddZoneDialog,
    SetDefaultZone,
}

#[derive(Debug)]
pub enum OutputSidebarMsg {
    ShowAddZoneDialog
}

pub struct SidebarView {
    broker: &'static FwdBroker,
    zones: FactoryVecDeque<ZoneItem>,
}

#[relm4::component(async, pub)]
impl SimpleAsyncComponent for SidebarView {
    type Init = ();
    type Input = InputSidebarMsg;
    type Output = OutputSidebarMsg;
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
                            sender.input(InputSidebarMsg::ShowAddZoneDialog);
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
            InputSidebarMsg::UpdateZones => {
                // let zones_names = self.broker.get_zones().await.unwrap_or_default();
                // self.zones = zones_names.into_iter().map(ZoneItem::from).collect();
                let zones = self.broker.get_zones().await.unwrap_or_default();
                for zone in zones {
                    self.zones.guard().push_back(zone);
                }
                sender.input(InputSidebarMsg::SetDefaultZone);
            }
            InputSidebarMsg::SetDefaultZone => {
                let default_zone = self.broker.get_default_zone().await.unwrap_or_default();
                println!("Default zone: {}", default_zone);
                for zone in self.zones.guard().iter_mut() {
                    zone.set_is_default(zone.name == default_zone);
                }
            }
            InputSidebarMsg::ShowAddZoneDialog => {
                let _ = sender.output(OutputSidebarMsg::ShowAddZoneDialog);
            }
        }
    }

    async fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let broker = FwdBroker::get_broker().await;

        let zones = FactoryVecDeque::builder()
            .launch_default()
            .forward(sender.input_sender(), |_| InputSidebarMsg::UpdateZones);
        
        let model = SidebarView {
            broker,
            zones
        };

        let zones_list_box = model.zones.widget();
        let widgets = view_output!();
        sender.input(InputSidebarMsg::UpdateZones);
        AsyncComponentParts { model, widgets }
    }
}
