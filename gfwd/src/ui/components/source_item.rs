use relm4::adw::prelude::*;
use relm4::prelude::*;

#[tracker::track]
#[derive(Debug, Clone, PartialEq)]
pub struct SourceItem {
    pub address: String,
    title: String,
    subtitle: String,
}

impl From<String> for SourceItem {
    fn from(address: String) -> Self {
        let title = address.clone();
        let subtitle = if address.contains('/') {
            "Network Range".to_string()
        } else if address.contains(':') {
            "IPv6 Address".to_string()
        } else {
            "IPv4 Address".to_string()
        };

        Self {
            address,
            title,
            subtitle,
            tracker: 0,
        }
    }
}

#[derive(Debug)]
pub enum SourceItemResponse {
    RemoveSource { address: String },
}

#[relm4::factory(pub)]
impl FactoryComponent for SourceItem {
    type Init = SourceItem;
    type Input = ();
    type Output = SourceItemResponse;
    type CommandOutput = ();
    type ParentWidget = gtk::ListBox;

    view! {
        adw::ActionRow {
            #[watch]
            set_title: &self.title,
            #[watch]
            set_subtitle: &self.subtitle,

            add_prefix = &gtk::Image {
                set_icon_name: Some("network-server-symbolic"),
                set_pixel_size: 16,
                add_css_class: "accent",
            },

            add_suffix = &gtk::Button {
                set_icon_name: "user-trash-symbolic",
                set_tooltip_text: Some("Remove source from zone"),
                add_css_class: "flat",
                add_css_class: "destructive-action",
                connect_clicked[sender, address = self.address.clone()] => move |_| {
                    sender.output(SourceItemResponse::RemoveSource {
                        address: address.clone(),
                    }).unwrap();
                },
            },
        }
    }

    fn init_model(init: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        init
    }

    fn update(&mut self, _message: Self::Input, _sender: FactorySender<Self>) {
        self.reset();
    }
}