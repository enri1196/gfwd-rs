use relm4::adw::prelude::*;
use relm4::prelude::*;

#[tracker::track]
#[derive(Debug)]
pub struct ZoneViewMode {
    pub(crate) zone_name: String,
}

#[derive(Debug)]
pub enum ZoneViewModeMsg {
    SetName(String)
}

#[relm4::component(pub)]
impl SimpleComponent for ZoneViewMode {
    type Init = String;
    type Input = ZoneViewModeMsg;
    type Output = ();

    view! {
        gtk::Label {
            #[track(model.changed(ZoneViewMode::zone_name()))]
            set_label: &format!("Viewing details for zone: {}", model.get_zone_name()),
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
        let model = ZoneViewMode { zone_name, tracker: 0 };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        self.reset();
        match message {
            ZoneViewModeMsg::SetName(zone_name) => self.set_zone_name(zone_name),
        }
    }
}
