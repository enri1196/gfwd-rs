use relm4::adw::prelude::*;
use relm4::prelude::*;

#[derive(Debug)]
pub enum ZoneViewMsg {
    ToggleSidebar,
    // Add more messages as needed
}

pub struct ZoneView {
}

#[relm4::component(pub)]
impl SimpleComponent for ZoneView {
    type Init = String; // Zone name
    type Input = ZoneViewMsg;
    type Output = ();

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            
            // Header Bar
            adw::HeaderBar {
                set_show_start_title_buttons: false,
                set_show_end_title_buttons: false,
                set_css_classes: &["flat"],
                
                // Toggle sidebar button on the left
                pack_start = &gtk::Button {
                    set_icon_name: "open-menu-symbolic",
                    connect_clicked[sender] => move |_| {
                        sender.output(()).unwrap();
                    },
                },
                
                // Zone title in the center
                #[wrap(Some)]
                set_title_widget = &gtk::Label::new(Some(&init)),
            },
            
            // Main content area
            gtk::ScrolledWindow {
                set_vexpand: true,
                set_hscrollbar_policy: gtk::PolicyType::Never,
                
                #[wrap(Some)]
                set_child = &gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_margin_all: 12,
                    set_spacing: 12,
                    
                    // Add your zone content here
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

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            ZoneViewMsg::ToggleSidebar => {
                // This will be handled by the parent component
            }
        }
    }
}
