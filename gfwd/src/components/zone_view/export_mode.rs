use relm4::adw::prelude::*;
use relm4::prelude::*;

#[derive(Debug)]
pub struct ZoneExportMode;

#[derive(Debug)]
pub enum ZoneExportModeMsg {}

#[relm4::component(pub)]
impl SimpleComponent for ZoneExportMode {
    type Init = ();
    type Input = ZoneExportModeMsg;
    type Output = ();

    view! {
        gtk::Label {
            set_label: "Export options for the zone would be here.",
            set_halign: gtk::Align::Center,
            set_valign: gtk::Align::Center,
            set_vexpand: true,
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = ZoneExportMode;
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }
}
