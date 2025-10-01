use relm4::adw::prelude::*;
use relm4::prelude::*;

#[tracker::track]
#[derive(Debug, Clone, PartialEq)]
pub struct InterfaceItem {
    pub name: String,
    title: String,
    subtitle: String,
}

impl From<String> for InterfaceItem {
    fn from(name: String) -> Self {
        let title = name.clone();
        let subtitle = "Network Interface".to_string();

        Self {
            name,
            title,
            subtitle,
            tracker: 0,
        }
    }
}

#[derive(Debug)]
pub enum InterfaceItemResponse {
    RemoveInterface { name: String },
}

#[relm4::factory(pub)]
impl FactoryComponent for InterfaceItem {
    type Init = InterfaceItem;
    type Input = ();
    type Output = InterfaceItemResponse;
    type CommandOutput = ();
    type ParentWidget = gtk::ListBox;

    view! {
        adw::ActionRow {
            #[watch]
            set_title: &self.title,
            #[watch]
            set_subtitle: &self.subtitle,

            add_prefix = &gtk::Image {
                set_icon_name: Some("network-wired-symbolic"),
                set_pixel_size: 16,
                add_css_class: "accent",
            },

            add_suffix = &gtk::Button {
                set_icon_name: "user-trash-symbolic",
                set_tooltip_text: Some("Remove interface from zone"),
                add_css_class: "flat",
                add_css_class: "destructive-action",
                connect_clicked[sender, name = self.name.clone()] => move |_| {
                    sender.output(InterfaceItemResponse::RemoveInterface {
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