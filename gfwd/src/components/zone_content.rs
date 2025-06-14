use relm4::actions::{AccelsPlus, RelmAction, RelmActionGroup};
use relm4::adw::prelude::*;
use relm4::prelude::*;

use crate::fwd_broker::FwdBroker;

#[tracker::track]
pub struct ZoneView {
    broker: &'static FwdBroker,
    current_zone_name: String
}

#[derive(Debug)]
pub enum ZoneViewRequest {
    SetZoneContent(String)
}

#[derive(Debug)]
pub enum ZoneViewResponse {
    ToggleSidebar
}

relm4::new_action_group!(WindowActionGroup, "win");

relm4::new_stateless_action!(DeleteZoneAction, WindowActionGroup, "example");

#[relm4::component(async, pub)]
impl SimpleAsyncComponent for ZoneView {
    type Init = String;
    type Input = ZoneViewRequest;
    type Output = ZoneViewResponse;

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,

            adw::HeaderBar {
                set_css_classes: &["flat"],

                pack_start = &gtk::Button {
                    set_icon_name: "sidebar-show-symbolic",
                    connect_clicked[sender] => move |_| {
                        sender.output(ZoneViewResponse::ToggleSidebar).unwrap();
                    },
                },

                pack_end = &gtk::MenuButton {
                    #[wrap(Some)]
                    set_popover = &gtk::PopoverMenu::from_model(Some(&main_menu)) {
                        // add_child: (&popover_child, "my_widget"),
                    }
                },

                #[wrap(Some)]
                set_title_widget = &gtk::Label {
                    #[track(model.changed(ZoneView::current_zone_name()))]
                    set_label: model.get_current_zone_name()
                }
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

    menu! {
        main_menu: {
            custom: "my_widget",
            "Delete Zone" => DeleteZoneAction,
            // "Example2" => ExampleAction,
            // "Example toggle" => ExampleU8Action(1_u8),
            // section! {
            //     "Section example" => ExampleAction,
            //     "Example toggle" => ExampleU8Action(1_u8),
            // },
            // section! {
            //     "Example" => ExampleAction,
            //     "Example2" => ExampleAction,
            //     "Example Value" => ExampleU8Action(1_u8),
            // },
            // "submenu1" {
            //     "Example" => ExampleAction,
            //     "Example2" => ExampleAction,
            //     "Example toggle" => ExampleU8Action(1_u8),
            //     "submenu2" {
            //         "Example" => ExampleAction,
            //         "Example2" => ExampleAction,
            //         "Example toggle" => ExampleU8Action(1_u8),
            //         "submenu3" {
            //             "Example" => ExampleAction,
            //             "Example2" => ExampleAction,
            //             "Example toggle" => ExampleU8Action(1_u8),
            //         }
            //     }
            // }
        }
    }

    async fn init(
        initial_zone_name: Self::Init,
        _root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let broker = FwdBroker::get_broker().await;
        let model = ZoneView {
            broker,
            current_zone_name: initial_zone_name,
            tracker: 0,
        };

        let widgets = view_output!();

        let app = relm4::main_application();
        app.set_accelerators_for_action::<DeleteZoneAction>(&["<primary>D"]);

        let action: RelmAction<DeleteZoneAction> = {
            RelmAction::new_stateless(move |_| {
                println!("Statelesss action!");
            })
        };

        let mut group = RelmActionGroup::<WindowActionGroup>::new();
        group.add_action(action);

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, msg: Self::Input, _sender: AsyncComponentSender<Self>) {
        match msg {
            ZoneViewRequest::SetZoneContent(zone_name) => self.set_current_zone_name(zone_name),
        }
    }
}
