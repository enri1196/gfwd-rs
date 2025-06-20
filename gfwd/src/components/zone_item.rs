use relm4::adw::prelude::*;
use relm4::gtk::glib::{self, LogLevel};
use relm4::prelude::*;

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

#[derive(Debug)]
pub enum ZoneItemResponse {
    /// Zone Item Name
    SelectedZone(String),
}

#[relm4::factory(pub)]
impl FactoryComponent for ZoneItem {
    type Init = String;
    type Input = ();
    type Output = ZoneItemResponse;
    type CommandOutput = ();
    type ParentWidget = gtk::ListBox;

    view! {
        gtk::ListBoxRow {
            set_halign: gtk::Align::Fill,
            set_margin_all: 4,

            gtk::Button {
                set_hexpand: true,

                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 8,
                    set_hexpand: true,

                    // Dot on the left if is_default
                    #[name(dot)]
                    gtk::Image {
                        set_pixel_size: 12,
                        set_icon_name: Some("media-record-symbolic"),
                        #[track(self.changed(ZoneItem::is_default()))]
                        set_visible: self.is_default,
                    },

                    // The label in the center
                    #[name(zone_label)]
                    gtk::Label {
                        #[watch]
                        set_label: &self.name,
                        set_hexpand: true,
                        set_halign: gtk::Align::Center,
                    },

                    // Tick on the right if is_active
                    #[name(tick)]
                    gtk::Image {
                        set_pixel_size: 16,
                        set_icon_name: Some("object-select-symbolic"),
                        #[track(self.changed(ZoneItem::is_active()))]
                        set_visible: self.is_active,
                    },
                },

                connect_clicked[sender, zone_label] => move |_| {
                    let zone_name = zone_label.label().to_string();
                    glib::g_log!(LogLevel::Message, "Selected zone {}", zone_name);
                    sender.output(ZoneItemResponse::SelectedZone(zone_name)).unwrap();
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

    fn update(&mut self, _message: Self::Input, _sender: FactorySender<Self>) {
        self.reset();
    }
}
