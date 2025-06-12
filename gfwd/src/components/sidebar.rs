use relm4::adw::prelude::*;
use relm4::prelude::*;
use crate::components::zone_dialog::{AddZoneDialog, AddZoneDialogOutput};
use crate::fwd_broker::FwdBroker;

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneItem {
    pub name: String,
    pub is_default: bool,
    pub is_active: bool,
    pub interfaces: Vec<String>,
}

impl From<String> for ZoneItem {
    fn from(value: String) -> Self {
        Self {
            name: value,
            is_default: false,
            is_active: false,
            interfaces: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub enum SidebarMsg {
    UpdateZones,
    ShowAddZoneDialog,
    ZoneAdded(AddZoneDialogOutput),
}

pub struct SidebarView {
    broker: &'static FwdBroker,
    dialog: AsyncController<AddZoneDialog>,
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
                set_width_request: 250,

                adw::HeaderBar {
                    set_show_end_title_buttons: false,
                    set_css_classes: &["flat"],

                    #[wrap(Some)]
                    set_title_widget = &gtk::Label {
                        set_text: "Firewall Zones",
                        set_css_classes: &["title-2"],
                        set_halign: gtk::Align::Start,
                    },

                    pack_end = &gtk::Button {
                        set_icon_name: "list-add-symbolic",
                        set_tooltip_text: Some("New Zone"),
                        set_css_classes: &["flat"],
                        connect_clicked[sender] => move |_| {
                            sender.input(SidebarMsg::ShowAddZoneDialog);
                        }
                    }
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

    async fn update(&mut self, msg: Self::Input, sender: AsyncComponentSender<Self>) {
        match msg {
            SidebarMsg::UpdateZones => {
                let zones_names = self.broker.get_zones().await.unwrap_or_default();
                self.zones = zones_names.into_iter().map(ZoneItem::from).collect();
            }
            SidebarMsg::ShowAddZoneDialog => {
                // Use sender.dialog to launch the modal and get its output
                self.dialog.widget().present(None::<&gtk::Box>);
            }
            SidebarMsg::ZoneAdded(output) => {
                if !output.name.is_empty() {
                    let _ = self.broker.add_zone(&output.name).await;
                    sender.input_sender().emit(SidebarMsg::UpdateZones);
                }
            }
        }
    }

    async fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let broker = FwdBroker::get_broker().await;
        let dialog = AddZoneDialog::builder()
            .launch(())
            .forward(sender.input_sender(), |msg| SidebarMsg::ZoneAdded(msg));

        let zones = broker.get_zones().await.unwrap_or_default();
        let model = SidebarView {
            broker,
            dialog,
            zones: zones.into_iter().map(ZoneItem::from).collect(),
        };

        let widgets = view_output!();
        for zone in model.zones.iter() {
            widgets
                .zones_list
                .append(&gtk::Label::new(Some(&zone.name)));
        }
        AsyncComponentParts { model, widgets }
    }
}
