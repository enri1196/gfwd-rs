use relm4::adw::prelude::*;
use relm4::prelude::*;

#[derive(Debug, PartialEq)]
pub struct PortItem {
    pub port: String,
    pub protocol: String,
}

#[relm4::factory(pub)]
impl FactoryComponent for PortItem {
    type Init = (String, String);
    type Input = ();
    type Output = ();
    type CommandOutput = ();
    type ParentWidget = gtk::ListBox;

    view! {
        adw::ActionRow {
            set_title: &self.port,
            set_subtitle: &self.protocol,
        }
    }

    fn init_model(
        init: Self::Init,
        _index: &DynamicIndex,
        _sender: FactorySender<Self>,
    ) -> Self {
        Self {
            port: init.0,
            protocol: init.1,
        }
    }
}
