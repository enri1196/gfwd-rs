use relm4::adw::prelude::*;
use relm4::prelude::*;

#[tracker::track]
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub struct IPSetItem {
    pub name: String,
    title: String,
    subtitle: String,
}

impl From<String> for IPSetItem {
    fn from(name: String) -> Self {
        let title = name.clone();
        let subtitle = "IP Set".to_string();

        Self {
            name,
            title,
            subtitle,
            tracker: 0,
        }
    }
}

#[derive(Debug)]
pub enum IPSetItemInput {
    Delete,
    Select,
}

#[derive(Debug)]
pub enum IPSetItemResponse {
    Delete(String),
    Select(String),
}

#[relm4::factory(pub)]
impl FactoryComponent for IPSetItem {
    type Init = String;
    type Input = IPSetItemInput;
    type Output = IPSetItemResponse;
    type CommandOutput = ();
    type ParentWidget = gtk::ListBox;

    view! {
        adw::ActionRow {
            #[watch]
            set_title: &self.title,
            #[watch]
            set_subtitle: &self.subtitle,
            set_activatable: true,

            add_prefix = &gtk::Image {
                set_icon_name: Some("network-server-symbolic"),
                add_css_class: "accent",
            },

            add_suffix = &gtk::Button {
                set_icon_name: "user-trash-symbolic",
                set_tooltip_text: Some("Delete IP set"),
                add_css_class: "flat",
                add_css_class: "destructive-action",
                set_valign: gtk::Align::Center,
                connect_clicked => IPSetItemInput::Delete,
            },

            connect_activated => IPSetItemInput::Select,
        }
    }

    fn init_model(name: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        name.into()
    }

    fn update(&mut self, msg: Self::Input, sender: FactorySender<Self>) {
        self.reset();

        match msg {
            IPSetItemInput::Delete => {
                let _ = sender.output(IPSetItemResponse::Delete(self.name.clone()));
            }
            IPSetItemInput::Select => {
                let _ = sender.output(IPSetItemResponse::Select(self.name.clone()));
            }
        }
    }
}
