use relm4::gtk::glib::{self, LogLevel};
use relm4::prelude::*;
use relm4::adw::prelude::*;

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
    SelectedZone(String)
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
                #[watch]
                set_label: &self.name,
                connect_clicked[sender] => move |btn| {
                    let zone_name = btn.label().map(|gs| gs.to_string()).unwrap_or("default".to_string());
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
}
