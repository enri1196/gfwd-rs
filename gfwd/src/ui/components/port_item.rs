use relm4::adw::prelude::*;
use relm4::prelude::*;

use crate::models::{ForwardingConfig, PortRule};

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
        let subtitle = if let Some(ref forward) = rule.forwarding {
            format!("→ {}:{}", forward.to_addr, forward.to_port)
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

            add_suffix = &gtk::Button {
                set_icon_name: "user-trash-symbolic",
                set_tooltip_text: Some("Remove port"),
                add_css_class: "flat",
                add_css_class: "destructive-action",
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
