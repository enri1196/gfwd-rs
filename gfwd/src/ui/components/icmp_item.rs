use relm4::adw::prelude::*;
use relm4::prelude::*;

use crate::models::IcmpType;

#[tracker::track]
#[derive(Debug, Clone, PartialEq)]
pub struct IcmpItem {
    pub name: String,
    pub description: String,
    title: String,
    subtitle: String,
}

impl From<IcmpType> for IcmpItem {
    fn from(icmp_type: IcmpType) -> Self {
        let title = icmp_type.name.clone();
        let subtitle = if icmp_type.description.is_empty() {
            "ICMP Type".to_string()
        } else {
            icmp_type.description
        };

        Self {
            name: icmp_type.name,
            description: subtitle.clone(),
            title,
            subtitle,
            tracker: 0,
        }
    }
}

impl From<String> for IcmpItem {
    fn from(name: String) -> Self {
        let title = name.clone();
        let subtitle = "ICMP Type".to_string();

        Self {
            name,
            description: subtitle.clone(),
            title,
            subtitle,
            tracker: 0,
        }
    }
}

#[derive(Debug)]
pub enum IcmpItemResponse {
    RemoveIcmp { name: String },
}

#[relm4::factory(pub)]
impl FactoryComponent for IcmpItem {
    type Init = IcmpItem;
    type Input = ();
    type Output = IcmpItemResponse;
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
                set_icon_name: Some("network-wired-symbolic"),
                set_pixel_size: 16,
                add_css_class: "accent",
                set_accessible_role: gtk::AccessibleRole::Img,
            },

            add_suffix = &gtk::Button {
                set_icon_name: "user-trash-symbolic",
                set_tooltip_text: Some("Remove ICMP block"),
                set_accessible_role: gtk::AccessibleRole::Button,
                set_can_focus: true,
                add_css_class: "flat",
                add_css_class: "destructive-action",
                connect_clicked[sender, name = self.name.clone()] => move |_| {
                    sender.output(IcmpItemResponse::RemoveIcmp {
                        name: name.clone(),
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
