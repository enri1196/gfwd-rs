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
        #[name(action_row)]
        adw::ActionRow {
            set_activatable: true,
            #[watch]
            set_title: &self.name,
            
            // Default zone indicator
            #[track(self.changed(ZoneItem::is_default()))]
            set_subtitle: if self.is_default { "Default Zone" } else { "" },
            
            // Icon prefix for default zone
            add_prefix = &gtk::Image {
                set_icon_name: Some("security-high-symbolic"),
                set_pixel_size: 16,
                #[track(self.changed(ZoneItem::is_default()))]
                set_visible: self.is_default,
                add_css_class: "accent",
            },

            // Active zone indicator suffix
            add_suffix = &gtk::Image {
                set_icon_name: Some("object-select-symbolic"),
                set_pixel_size: 16,
                #[track(self.changed(ZoneItem::is_active()))]
                set_visible: self.is_active,
                add_css_class: "success",
            },

            connect_activated[sender] => move |row| {
                let zone_name = row.title().to_string();
                glib::g_log!(LogLevel::Message, "Selected zone {}", zone_name);
                sender.output(ZoneItemResponse::SelectedZone(zone_name)).unwrap();
            },
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