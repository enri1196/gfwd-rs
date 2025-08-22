use relm4::adw::prelude::*;
use relm4::prelude::*;

use crate::components::zone_view::view_mode::ForwardOpts;

#[derive(Debug, PartialEq)]
pub struct PortItem {
    pub port: String,
    pub protocol: String,
    pub forwarding_port: Option<ForwardOpts>,
}

#[derive(Debug)]
pub enum PortItemInput {
    Remove,
}

#[relm4::factory(pub)]
impl FactoryComponent for PortItem {
    type Init = (String, String, Option<ForwardOpts>);
    type Input = PortItemInput;
    type Output = (String, String);
    type CommandOutput = ();
    type ParentWidget = gtk::ListBox;

    view! {
        adw::ActionRow {
            #[watch]
            set_title: &if let Some(ref forward) = self.forwarding_port {
                format!("{} ({}) → {}:{}", self.port, self.protocol, forward.to_addr, forward.to_port)
            } else {
                format!("{} ({})", self.port, self.protocol)
            },

            #[watch]
            set_subtitle: &if self.forwarding_port.is_some() {
                "Forwarded Port".to_string()
            } else {
                "Allowed Port".to_string()
            },

            add_prefix = &gtk::Image {
                #[watch]
                set_icon_name: Some(if self.forwarding_port.is_some() {
                    "network-transmit-receive-symbolic"
                } else {
                    "network-server-symbolic"
                }),
                set_pixel_size: 16,
                set_margin_end: 6,
            },

            add_suffix = &gtk::Button {
                set_icon_name: "user-trash-symbolic",
                set_tooltip_text: Some("Remove port"),
                set_valign: gtk::Align::Center,
                set_vexpand: false,
                set_margin_top: 6,
                set_margin_bottom: 6,
                connect_clicked[sender, _port = self.port.clone(), _protocol = self.protocol.clone()] => move |_| {
                    sender.input(PortItemInput::Remove);
                }
            }
        }
    }

    fn init_model(init: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        Self {
            port: init.0,
            protocol: init.1,
            forwarding_port: init.2
        }
    }

    fn update(&mut self, message: Self::Input, sender: FactorySender<Self>) {
        match message {
            PortItemInput::Remove => {
                let _ = sender.output((self.port.clone(), self.protocol.clone()));
            }
        }
    }
}
