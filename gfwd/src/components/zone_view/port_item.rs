use relm4::adw::prelude::*;
use relm4::prelude::*;

#[derive(Debug, PartialEq)]
pub struct PortItem {
    pub port: String,
    pub protocol: String,
}

#[derive(Debug)]
pub enum PortItemInput {
    Remove,
}

#[relm4::factory(pub)]
impl FactoryComponent for PortItem {
    type Init = (String, String);
    type Input = PortItemInput;
    type Output = (String, String);
    type CommandOutput = ();
    type ParentWidget = gtk::ListBox;

    view! {
        adw::ActionRow {
            set_title: &self.port,
            set_subtitle: &self.protocol,

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
