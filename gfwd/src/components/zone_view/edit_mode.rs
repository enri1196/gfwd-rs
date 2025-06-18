use relm4::adw::prelude::*;
use relm4::prelude::*;

#[derive(Debug)]
pub struct ZoneEditMode {
    pub(crate) zone_name: String,
}

#[derive(Debug)]
pub enum ZoneEditModeMsg {}

#[relm4::component(pub)]
impl SimpleComponent for ZoneEditMode {
    type Init = String;
    type Input = ZoneEditModeMsg;
    type Output = ();

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 6,
            set_halign: gtk::Align::Center,
            set_valign: gtk::Align::Center,
            set_vexpand: true,

            gtk::Label {
                set_label: "You are now in Edit mode.",
            },
            gtk::Entry {
                set_text: &model.zone_name,
            }
        }
    }

    fn init(
        zone_name: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = ZoneEditMode { zone_name };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }
}
