use relm4::adw::prelude::*;
use relm4::prelude::*;

#[derive(Debug)]
pub struct ZoneViewMode {
    pub(crate) zone_name: String,
}

#[derive(Debug)]
pub enum ZoneViewModeMsg {}

#[relm4::component(pub)]
impl SimpleComponent for ZoneViewMode {
    type Init = String;
    type Input = ZoneViewModeMsg;
    type Output = ();

    view! {
        gtk::Label {
            set_label: &format!("Viewing details for zone: {}", model.zone_name),
            set_halign: gtk::Align::Center,
            set_valign: gtk::Align::Center,
            set_vexpand: true,
        }
    }

    fn init(
        zone_name: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = ZoneViewMode { zone_name };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }
}
