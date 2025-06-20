use relm4::adw::prelude::*;
use relm4::prelude::*;

#[tracker::track]
#[derive(Debug)]
pub struct ZoneExportMode {
    zone_name: String
}

#[derive(Debug)]
pub enum ZoneExportModeMsg {
    SetName(String)
}

#[relm4::component(pub)]
impl SimpleComponent for ZoneExportMode {
    type Init = String;
    type Input = ZoneExportModeMsg;
    type Output = ();

    view! {
        gtk::Label {
            #[track(model.changed(ZoneExportMode::zone_name()))]
            set_label: &format!("Export options for '{}' zone would be here.", model.get_zone_name()),
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
        let model = ZoneExportMode { zone_name, tracker: 0 };
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        self.reset();
        match message {
            ZoneExportModeMsg::SetName(zone_name) => self.set_zone_name(zone_name),
        }
    }
}
