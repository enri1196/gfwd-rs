mod edit_mode;
mod export_mode;
mod view_mode;

use relm4::actions::{AccelsPlus, RelmAction, RelmActionGroup};
use relm4::adw::prelude::*;
use relm4::prelude::*;

use crate::fwd_broker::FwdBroker;
use edit_mode::ZoneEditMode;
use export_mode::ZoneExportMode;
use view_mode::ZoneViewMode;

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub enum ActiveView {
    #[default]
    View,
    Edit,
    Export,
}

#[tracker::track]
pub struct ZoneView {
    #[tracker::do_not_track]
    broker: &'static FwdBroker,
    current_zone_name: String,
    active_view: ActiveView,
    // Child components for each view mode
    #[tracker::do_not_track]
    view_mode: Controller<ZoneViewMode>,
    #[tracker::do_not_track]
    edit_mode: Controller<ZoneEditMode>,
    #[tracker::do_not_track]
    export_mode: Controller<ZoneExportMode>,
}

/// Requests that can be sent to the ZoneView component.
#[derive(Debug)]
pub enum ZoneViewRequest {
    SetZoneContent(String),
    SwitchTo(ActiveView),
}

/// Responses that can be emitted from the ZoneView component.
#[derive(Debug)]
pub enum ZoneViewResponse {
    ToggleSidebar,
}

relm4::new_action_group!(WindowActionGroup, "win");
relm4::new_stateless_action!(DeleteZoneAction, WindowActionGroup, "example");

#[relm4::component(async, pub)]
impl AsyncComponent for ZoneView {
    type Init = String;
    type Input = ZoneViewRequest;
    type Output = ZoneViewResponse;
    type CommandOutput = ();

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
                    set_popover = &gtk::PopoverMenu::from_model(Some(&main_menu)) {}
                },

                #[wrap(Some)]
                set_title_widget = &gtk::Box {
                    add_css_class: "linked",
                    #[name = "group"]
                    gtk::ToggleButton {
                        set_label: "View",
                        set_active: model.active_view == ActiveView::View,
                        connect_toggled[sender] => move |btn| {
                            if btn.is_active() {
                                sender.input(ZoneViewRequest::SwitchTo(ActiveView::View));
                            }
                        },
                    },
                    gtk::ToggleButton {
                        set_label: "Edit",
                        set_group: Some(&group),
                        set_active: model.active_view == ActiveView::Edit,
                        connect_toggled[sender] => move |btn| {
                            if btn.is_active() {
                                sender.input(ZoneViewRequest::SwitchTo(ActiveView::Edit));
                            }
                        },
                    },
                    gtk::ToggleButton {
                        set_label: "Export",
                        set_group: Some(&group),
                        set_active: model.active_view == ActiveView::Export,
                        connect_toggled[sender] => move |btn| {
                            if btn.is_active() {
                                sender.input(ZoneViewRequest::SwitchTo(ActiveView::Export));
                            }
                        },
                    },
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

                    // Conditionally show the child components based on the active view
                    match model.active_view {
                        ActiveView::View => model.view_mode.widget().clone(),
                        ActiveView::Edit => model.edit_mode.widget().clone(),
                        ActiveView::Export => model.export_mode.widget().clone(),
                    }
                },
            },
        }
    }

    menu! {
        main_menu: {
            "Delete Zone" => DeleteZoneAction,
        }
    }

    async fn init(
        initial_zone_name: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let broker = FwdBroker::get_broker().await;

        // Initialize child components
        let view_mode = ZoneViewMode::builder()
            .launch(initial_zone_name.clone())
            .forward(sender.input_sender(), |_| unimplemented!());

        let edit_mode = ZoneEditMode::builder()
            .launch(initial_zone_name.clone())
            .forward(sender.input_sender(), |_| unimplemented!());

        let export_mode = ZoneExportMode::builder()
            .launch(())
            .forward(sender.input_sender(), |_| unimplemented!());


        let model = ZoneView {
            broker,
            current_zone_name: initial_zone_name,
            active_view: ActiveView::default(),
            view_mode,
            edit_mode,
            export_mode,
            tracker: 0,
        };

        let widgets = view_output!();

        let app = relm4::main_application();
        app.set_accelerators_for_action::<DeleteZoneAction>(&["<primary>D"]);

        let action: RelmAction<DeleteZoneAction> = {
            RelmAction::new_stateless(move |_| {
                println!("Statelesss action for deleting zone!");
            })
        };

        let mut group = RelmActionGroup::<WindowActionGroup>::new();
        group.add_action(action);
        root.insert_action_group("win", Some(&group.into_action_group()));

        AsyncComponentParts { model, widgets }
    }

    async fn update(
        &mut self,
        msg: Self::Input,
        _sender: AsyncComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match msg {
            ZoneViewRequest::SetZoneContent(zone_name) => {
                self.set_current_zone_name(zone_name);
                // Potentially send updates to child components here if needed
            }
            ZoneViewRequest::SwitchTo(view) => {
                self.set_active_view(view);
            }
        }
    }
}
