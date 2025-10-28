use relm4::adw::prelude::*;
use relm4::prelude::*;

use crate::models::{ForwardingConfig, PortRule};
use crate::ui::styling::{css_classes, icons};

#[tracker::track]
#[derive(Debug, Clone, PartialEq)]
pub struct PortItem {
    pub port: String,
    pub protocol: String,
    pub forwarding: Option<ForwardingConfig>,
    title: String,
    subtitle: String,
}

impl From<PortRule> for PortItem {
    fn from(rule: PortRule) -> Self {
        let title = format!("{}/{}", rule.port, rule.protocol);
        let subtitle = if rule.is_forwarded() {
            if let Some(ref forward) = rule.forwarding {
                format!("→ {}:{}", forward.to_addr, forward.to_port)
            } else {
                "Forwarded Port".to_string()
            }
        } else {
            "Allowed Port".to_string()
        };

        Self {
            port: rule.port,
            protocol: rule.protocol,
            forwarding: rule.forwarding,
            title,
            subtitle,
            tracker: 0,
        }
    }
}

impl From<(String, String)> for PortItem {
    fn from((port, protocol): (String, String)) -> Self {
        let title = format!("{}/{}", port, protocol);
        let subtitle = "Allowed Port".to_string();

        Self {
            port,
            protocol,
            forwarding: None,
            title,
            subtitle,
            tracker: 0,
        }
    }
}

impl From<(String, String, String, String)> for PortItem {
    fn from((port, protocol, to_port, to_addr): (String, String, String, String)) -> Self {
        let title = format!("{}/{}", port, protocol);
        let subtitle = format!("→ {}:{}", to_addr, to_port);
        let forwarding = Some(ForwardingConfig { to_port, to_addr });

        Self {
            port,
            protocol,
            forwarding,
            title,
            subtitle,
            tracker: 0,
        }
    }
}

#[derive(Debug)]
pub enum PortItemResponse {
    RemovePort {
        port: String,
        protocol: String,
        forwarding: Option<ForwardingConfig>,
    },
}

#[relm4::factory(pub)]
impl FactoryComponent for PortItem {
    type Init = PortItem;
    type Input = ();
    type Output = PortItemResponse;
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
                set_icon_name: Some(if self.forwarding.is_some() {
                    icons::GO_JUMP
                } else {
                    icons::NETWORK_WIRED
                }),
                set_pixel_size: 16,
                set_accessible_role: gtk::AccessibleRole::Img,
                add_css_class: if self.forwarding.is_some() {
                    css_classes::ACCENT
                } else {
                    css_classes::SUCCESS
                },
            },

            add_suffix = &gtk::Button {
                set_icon_name: icons::REMOVE,
                set_tooltip_text: Some("Remove port rule"),
                set_accessible_role: gtk::AccessibleRole::Button,
                set_can_focus: true,
                add_css_class: css_classes::FLAT,
                add_css_class: css_classes::DESTRUCTIVE_ACTION,
                connect_clicked[sender, port = self.port.clone(), protocol = self.protocol.clone(), forwarding = self.forwarding.clone()] => move |_| {
                    sender.output(PortItemResponse::RemovePort {
                        port: port.clone(),
                        protocol: protocol.clone(),
                        forwarding: forwarding.clone(),
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
