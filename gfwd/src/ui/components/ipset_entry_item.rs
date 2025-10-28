use relm4::adw::prelude::*;
use relm4::prelude::*;

#[tracker::track]
#[derive(Debug, Clone, PartialEq)]
pub struct IPSetEntryItem {
    pub entry: String,
    title: String,
    subtitle: String,
}

impl From<String> for IPSetEntryItem {
    fn from(entry: String) -> Self {
        let subtitle = if entry.contains(',') {
            "Combined entry".to_string()
        } else if entry.contains('/') {
            "Network entry".to_string()
        } else if entry.chars().filter(|c| *c == ':').count() == 5 && !entry.contains("::") {
            "MAC address".to_string()
        } else if entry.contains(':') {
            "IPv6 address".to_string()
        } else {
            "IPv4 address or value".to_string()
        };

        Self {
            title: entry.clone(),
            entry,
            subtitle,
            tracker: 0,
        }
    }
}

#[derive(Debug)]
pub enum IPSetEntryItemResponse {
    RemoveEntry { entry: String },
}

#[relm4::factory(pub)]
impl FactoryComponent for IPSetEntryItem {
    type Init = String;
    type Input = ();
    type Output = IPSetEntryItemResponse;
    type CommandOutput = ();
    type ParentWidget = gtk::ListBox;

    view! {
        adw::ActionRow {
            #[watch]
            set_title: &self.title,
            #[watch]
            set_subtitle: &self.subtitle,
            set_accessible_role: gtk::AccessibleRole::ListItem,

            add_prefix = &gtk::Image {
                set_icon_name: Some("network-server-symbolic"),
                set_pixel_size: 16,
                add_css_class: "accent",
                set_accessible_role: gtk::AccessibleRole::Img,
            },

            add_suffix = &gtk::Button {
                set_icon_name: "user-trash-symbolic",
                set_tooltip_text: Some("Remove entry from IP set"),
                add_css_class: "flat",
                add_css_class: "destructive-action",
                set_accessible_role: gtk::AccessibleRole::Button,
                set_can_focus: true,
                connect_clicked[sender, entry = self.entry.clone()] => move |_| {
                    sender.output(IPSetEntryItemResponse::RemoveEntry { entry: entry.clone() }).unwrap();
                },
            },
        }
    }

    fn init_model(entry: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        entry.into()
    }

    fn update(&mut self, _msg: Self::Input, _sender: FactorySender<Self>) {
        self.reset();
    }
}
