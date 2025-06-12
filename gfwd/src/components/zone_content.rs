use relm4::adw::prelude::*;
use relm4::prelude::*;

pub struct ZoneView {}

#[relm4::component(pub)]
impl SimpleComponent for ZoneView {
    type Init = String;
    type Input = ();
    type Output = ();

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,

            adw::HeaderBar {
                set_css_classes: &["flat"],

                pack_start = &gtk::Button {
                    set_icon_name: "open-menu-symbolic",
                    connect_clicked[sender] => move |_| {
                        sender.output(()).unwrap();
                    },
                },

                #[wrap(Some)]
                set_title_widget = &gtk::Label::new(Some(&init)),
            },

            gtk::ScrolledWindow {
                set_vexpand: true,
                set_hscrollbar_policy: gtk::PolicyType::Never,

                #[wrap(Some)]
                set_child = &gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_margin_all: 12,
                    set_spacing: 12,

                    gtk::Label {
                        set_label: "Zone details will be shown here",
                        set_halign: gtk::Align::Center,
                        set_valign: gtk::Align::Center,
                    },
                },
            },
        }
    }

    fn init(
        init: Self::Init,
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let widgets = view_output!();
        let model = ZoneView {};
        ComponentParts { model, widgets }
    }

    fn update(&mut self, _msg: Self::Input, _sender: ComponentSender<Self>) {}
}
