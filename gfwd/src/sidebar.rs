use relm4::adw::prelude::*;
use relm4::prelude::*;
use crate::models::sidebar::{SidebarModel, Zone};

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneItem {
    pub name: String,
    pub is_default: bool,
    pub is_active: bool,
    pub interfaces: Vec<String>,
}

impl From<&Zone> for ZoneItem {
    fn from(zone: &Zone) -> Self {
        Self {
            name: zone.name.clone(),
            is_default: zone.is_default,
            is_active: zone.is_active,
            interfaces: zone.interfaces.clone(),
        }
    }
}

// #[derive(Debug)]
// pub struct SidebarWidgets {
//     zones_list: gtk::ListBox,
// }

#[derive(Debug, PartialEq)]
pub enum SidebarMsg {
    UpdateZones,
}

#[derive(Debug)]
pub struct SidebarView {
    sb_model: SidebarModel,
    pub width: i32,
    zones: Vec<ZoneItem>,
}

#[relm4::component(async, pub)]
impl SimpleAsyncComponent for SidebarView {
    type Init = ();
    type Input = SidebarMsg;
    type Output = ();
    type Widgets = SidebarWidgets;

    view! {
        gtk::ScrolledWindow {
            set_vexpand: true,
            set_hscrollbar_policy: gtk::PolicyType::Never,
            #[wrap(Some)]
            set_child = &gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 12,
                set_margin_all: 12,
                set_width_request: model.width,

                append = &gtk::Label {
                    set_text: "Firewall Zones",
                    set_css_classes: &["title-2"],
                    set_halign: gtk::Align::Start,
                    set_margin_bottom: 12,
                },

                append = zones_list = &gtk::ListBox {
                    set_selection_mode: gtk::SelectionMode::None,
                    set_css_classes: &["rich-list", "boxed-list"],
                    set_show_separators: true,
                    set_valign: gtk::Align::Start,
                },
            }
        }
    }

    async fn update(&mut self, msg: Self::Input, _sender: AsyncComponentSender<Self>) {
        match msg {
            SidebarMsg::UpdateZones => {
                self.zones = self.sb_model.get_zones().await
                    .iter().map(ZoneItem::from).collect();
            }
        }
    }

    async fn init(
        _init: Self::Init,
        root: Self::Root,
        _sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let sb_model = SidebarModel::new().await;
        let zones = sb_model.get_zones().await;
        let model = SidebarView {
            sb_model,
            width: 250,
            zones: zones.iter().map(ZoneItem::from).collect(),
        };

        let widgets = view_output!();
        for zone in model.zones.iter() {
            widgets.zones_list.append(&gtk::Label::new(Some(&zone.name)));
        }
        AsyncComponentParts { model, widgets }
    }
}
